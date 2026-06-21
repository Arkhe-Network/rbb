use ndarray::{s, Array2};
use ndarray_rand::RandomExt;
use rand::distributions::Standard;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub(crate) data: Array2<f32>,
}

impl Tensor {
    pub fn zeros(shape: (usize, usize)) -> Self {
        Self {
            data: Array2::zeros(shape),
        }
    }

    pub fn randn(shape: (usize, usize)) -> Self {
        let mut rng = StdRng::seed_from_u64(42);
        Self {
            data: Array2::random_using(shape, Standard, &mut rng),
        }
    }

    pub fn shape(&self) -> (usize, usize) {
        let dims = self.data.shape();
        (dims[0], dims[1])
    }

    pub fn nrows(&self) -> usize {
        self.data.shape()[0]
    }
    pub fn ncols(&self) -> usize {
        self.data.shape()[1]
    }

    pub fn slice_row(&self, idx: usize) -> Tensor {
        Tensor {
            data: self.data.slice(s![idx..idx + 1, ..]).to_owned(),
        }
    }

    pub fn scale(&self, scalar: f32) -> Tensor {
        Tensor {
            data: &self.data * scalar,
        }
    }

    pub fn matmul(&self, other: &Tensor) -> Tensor {
        Tensor {
            data: self.data.dot(&other.data),
        }
    }

    pub fn mapv(&self, f: impl Fn(f32) -> f32) -> Tensor {
        Tensor {
            data: self.data.mapv(f),
        }
    }

    pub fn clamp(&self, min: f32, max: f32) -> Tensor {
        self.mapv(|v| v.clamp(min, max))
    }

    pub fn sigmoid(&self) -> Tensor {
        self.mapv(|v| 1.0 / (1.0 + (-v).exp()))
    }

    pub fn sum(&self) -> f32 {
        self.data.sum()
    }

    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.data[[i, j]]
    }
    pub fn set(&mut self, i: usize, j: usize, val: f32) {
        self.data[[i, j]] = val;
    }
    pub fn transpose(&self) -> Tensor {
        Tensor {
            data: self.data.t().to_owned(),
        }
    }

    pub fn to_vec(&self) -> Vec<f32> {
        self.data.iter().copied().collect()
    }
}

impl Add<&Tensor> for &Tensor {
    type Output = Tensor;
    fn add(self, other: &Tensor) -> Tensor {
        Tensor {
            data: &self.data + &other.data,
        }
    }
}

impl Add<&Tensor> for Tensor {
    type Output = Tensor;
    fn add(self, other: &Tensor) -> Tensor {
        Tensor {
            data: self.data + &other.data,
        }
    }
}

impl Sub<&Tensor> for &Tensor {
    type Output = Tensor;
    fn sub(self, other: &Tensor) -> Tensor {
        Tensor {
            data: &self.data - &other.data,
        }
    }
}

impl Mul<&Tensor> for &Tensor {
    type Output = Tensor;
    fn mul(self, other: &Tensor) -> Tensor {
        Tensor {
            data: &self.data * &other.data,
        }
    }
}

impl From<Array2<f32>> for Tensor {
    fn from(data: Array2<f32>) -> Self {
        Self { data }
    }
}
