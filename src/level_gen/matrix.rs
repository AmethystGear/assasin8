/// generic N dimensional matrix that can contain any types implementing
/// the Copy and Default traits.
pub struct Matrix<T, const N : usize> where T : Copy + Default {
    dim : [usize; N],
    elems : Vec<T>
}

impl <T, const N: usize> Matrix<T, N> where T : Copy + Default {
    /// construct a new matrix with the provided dimensionality
    /// and fill it with T::default().
    pub fn new(dim : [usize; N]) -> Self {
        let size = dim.iter().fold(1, |acc, elem| acc * elem);
        Self {
            dim,
            elems : vec![T::default(); size]
        }
    }

    fn index(&self, loc : [usize; N]) -> usize {
        let mut index = loc[0];
        for i in 1..N {
            index += self.dim[i - 1] * loc[i];
        }
        index
    }

    /// get the element at the provided location.
    /// you must ensure that the location is within the bounds,
    /// otherwise the function may return the wrong T, or panic.
    pub fn get(&self, loc : [usize; N]) -> T {
        self.elems[self.index(loc)]
    }

    /// set the element at the provided location.
    /// you must ensure that the location is within the bounds,
    /// otherwise the function may set the wrong element, or panic.
    pub fn set(&mut self, loc : [usize; N], elem : T)  {
        let index = self.index(loc);
        self.elems[index] = elem;
    }

    /// returns the dimensions of the Matrix
    pub fn dim(&self) -> [usize; N] {
        self.dim
    }
}