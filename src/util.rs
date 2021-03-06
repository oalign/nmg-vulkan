#![allow(dead_code)] // Library

use std;

pub struct AlignedBuffer<T> {
    alignment: usize, // Alignment in usizes (not bytes)
    length: usize, // Length in usizes (not bytes)
    start: *const T,
    ptr: *mut usize,
}

impl<T> AlignedBuffer<T> {
    pub fn new(
        alignment: usize, // Bytes
        count:     usize,
    ) -> AlignedBuffer<T> {
        debug_assert!(count != 0);

        let ptr_len = std::mem::size_of::<usize>();
        debug_assert!(alignment >= ptr_len);
        debug_assert!(std::mem::size_of::<T>() <= alignment);

        let alignment = alignment / ptr_len;
        let length = count * alignment;

        // Waiting for std::heap...
        let mut memory = Vec::<usize>::with_capacity(length);
        let ptr = memory.as_mut_ptr();
        std::mem::forget(memory);

        let start = ptr as *const T;

        AlignedBuffer {
            alignment,
            length,
            start,
            ptr,
        }
    }

    // Returns alignment of buffer in bytes
    pub fn byte_alignment(&self) -> usize {
        self.alignment * std::mem::size_of::<usize>()
    }

    // Returns size of buffer in bytes
    pub fn size(&self) -> usize {
        self.length * std::mem::size_of::<usize>()
    }

    pub fn push(&mut self, entry: T) {
        assert!(
            (self.ptr as usize - self.start as usize)
                / std::mem::size_of::<usize>()
                < self.length
        );

        unsafe {
            std::ptr::copy_nonoverlapping(
                &entry as *const T,
                self.ptr as *mut T,
                1,
            );

            self.ptr = self.ptr.offset(self.alignment as isize);
        }
    }

    pub unsafe fn finalize(&self) -> Vec<usize> {
        Vec::from_raw_parts(
            self.start as *mut usize,
            self.length,
            self.length,
        )
    }
}

#[cfg(test)]
mod tests {
    use alg;
    use render;
    use util::*;

    #[test]
    fn pack_ubo() {
        let mat = alg::Mat4::id();

        let make_offset = |vec| render::PaddedVec3::new(vec);

        let offsets = [
            make_offset(alg::Vec3::new(0., 0.5, 0.)),
            make_offset(alg::Vec3::new(0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(0., 0.5, 0.)),
            make_offset(alg::Vec3::new(0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(0., 0.5, 0.)),
            make_offset(alg::Vec3::new(0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(0., 0.5, 0.)),
            make_offset(alg::Vec3::new(0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(0., 0.5, 0.)),
            make_offset(alg::Vec3::new(0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, -0.5)),
            make_offset(alg::Vec3::new(0.5, -0.5, 0.5)),
            make_offset(alg::Vec3::new(-0.5, -0.5, 0.5)),
        ];

        let mut raw = {
            let mut buffer = AlignedBuffer::new(996, 1);

            buffer.push(
                render::InstanceUBO::new(
                    mat,
                    [render::Light::default(); render::MAX_INSTANCE_LIGHTS],
                    offsets,
                    [render::PaddedVec3::default(); render::MAX_SOFTBODY_VERT],
                )
            );

            unsafe {
                buffer.finalize()
            }
        };

        let (test_mat, test_offsets) = unsafe {
            let mut ptr = raw.as_mut_ptr() as *const alg::Mat4;
            let test_mat = *ptr;

            ptr = ptr.offset(1);
            let mut ptr = ptr as *const render::Light;
            ptr = ptr.offset(render::MAX_INSTANCE_LIGHTS as isize);

            let mut ptr = ptr as *const render::PaddedVec3;
            let mut test_offsets = Vec::with_capacity(offsets.len());

            for _ in 0..offsets.len() {
                let offset = *ptr;
                test_offsets.push(offset);
                ptr = ptr.offset(1);
            }

            (test_mat, test_offsets)
        };

        assert!(test_mat == mat);

        for i in 0..offsets.len() {
            assert!(test_offsets[i] == offsets[i]);
        }
    }

    #[test]
    fn create_aligned_buffers() {
        let matrices = [
            alg::Mat4::id(),
            alg::Mat4::translation(-1., 2., 5.),
            alg::Mat4::translation(8., 3., 3.),
        ];

        compare_aligned_buffer(64, &matrices);
        compare_aligned_buffer(128, &matrices);
        compare_aligned_buffer(256, &matrices);
        compare_aligned_buffer(512, &matrices);
    }

    fn compare_aligned_buffer(alignment: usize, matrices: &[alg::Mat4]) {
        let mut raw = {
            let mut buffer = AlignedBuffer::new(alignment, matrices.len());

            for &matrix in matrices {
                buffer.push(matrix);
            }

            unsafe {
                buffer.finalize()
            }
        };

        let aligned = {
            let mut result = Vec::<alg::Mat4>::with_capacity(matrices.len());
            let ptr_len = std::mem::size_of::<usize>();

            unsafe {
                let mut ptr = raw.as_mut_ptr();
                let offset = (alignment / ptr_len) as isize;
                let mut start = ptr as usize;

                for i in 0..matrices.len() {
                    let matrix = *(ptr as *const alg::Mat4);
                    result.push(matrix);

                    let diff = {
                        let end = ptr as usize;
                        let diff = (end - start) / ptr_len;
                        start = end;

                        diff
                    };

                    eprintln!(
                        "\n\tmatrix[{}] diff = {} ({}B)\n{}",
                        i,
                        diff,
                        diff * ptr_len,
                        matrix,
                    );

                    if i > 0 {
                        assert!(diff as isize == offset);
                    }

                    ptr = ptr.offset(offset);
                }
            }

            result
        };

        // Compare matrices
        for i in 0..aligned.len() {
            assert!(aligned[i] == matrices[i]);
        }
    }
}
