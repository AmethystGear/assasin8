use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{self, RenderGraph},
        render_resource::{BindGroup, ComputePassDescriptor, PipelineCache},
        RenderApp, RenderSet,
    },
    window::PrimaryWindow,
};

use super::{
    pipeline::{
        extract_pipeline_assets, prepare_pipeline_assets, ShadowPipeline, ShadowPipelineAssets, queue_bind_group, setup_pipeline,
    },
    types::{LightData, OcclusionData},
};

pub struct ShadowRenderPass;

#[derive(Default)]
struct ShadowRenderNode;

const WORKGROUP_SIZE: u32 = 8;
impl render_graph::Node for ShadowRenderNode {
    fn run(
        &self,
        _: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if let Some(pipeline_bind_group) = world.get_resource::<ShadowRenderBindGroup>() {
            let shadow_pipeline = world.resource::<ShadowPipeline>();
            let pipeline_cache = world.resource::<PipelineCache>();
            let target_sizes = world.resource::<ComputedTargetSizes>();
            if let Some(shadow_pipeline) =
                pipeline_cache.get_compute_pipeline(shadow_pipeline.pipeline_id)
            {
                let w = target_sizes.primary_target_usize.x;
                let h = target_sizes.primary_target_usize.y;
                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("shadow_pass_2D"),
                        });

                pass.set_bind_group(0, &pipeline_bind_group.bind_group, &[]);
                pass.set_pipeline(shadow_pipeline);
                pass.dispatch_workgroups(w / WORKGROUP_SIZE, h / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}

#[derive(Default, Resource, Copy, Clone)]
pub struct ComputedTargetSizes {
    pub(crate) primary_target_size: Vec2,
    pub(crate) primary_target_isize: IVec2,
    pub(crate) primary_target_usize: UVec2,
}

impl Plugin for ShadowRenderPass {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<LightData>::default())
            .add_plugin(UniformComponentPlugin::<LightData>::default())
            .add_plugin(ExtractComponentPlugin::<OcclusionData>::default())
            .add_plugin(UniformComponentPlugin::<OcclusionData>::default())
            .init_resource::<ComputedTargetSizes>()
            .add_startup_system(detect_target_sizes)
            .add_startup_system(setup_pipeline.after(detect_target_sizes));
        //.add_startup_system(setup_post_processing_camera.after(system_setup_gi_pipeline));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ShadowPipeline>()
            .init_resource::<ShadowPipelineAssets>()
            .init_resource::<ComputedTargetSizes>()
            .add_system(extract_pipeline_assets.in_schedule(ExtractSchedule))
            .add_system(prepare_pipeline_assets.in_set(RenderSet::Prepare))
            .add_system(queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("shadow_render", ShadowRenderNode::default());
        render_graph.add_node_edge(
            "shadow_render",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }
}

fn detect_target_sizes(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut target_sizes: ResMut<ComputedTargetSizes>,
) {
    let window = windows.get_single().expect("No primary window");
    let primary_size = Vec2::new(
        (window.physical_width() as f64 / window.scale_factor()) as f32,
        (window.physical_height() as f64 / window.scale_factor()) as f32,
    );

    target_sizes.primary_target_size = primary_size;
    target_sizes.primary_target_isize = target_sizes.primary_target_size.as_ivec2();
    target_sizes.primary_target_usize = target_sizes.primary_target_size.as_uvec2();
}

#[derive(Resource)]
pub struct ShadowRenderBindGroup {
    pub bind_group: BindGroup,
}
