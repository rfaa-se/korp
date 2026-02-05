pub(super) struct Uniform {
    pub(super) buffer: wgpu::Buffer,
    pub(super) bind_group: wgpu::BindGroup,
    pub(super) bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub(super) struct UniformBuffer {
    pub(super) view_projection: [[f32; 4]; 4],
}

impl Uniform {
    pub(super) fn new(device: &mut wgpu::Device, view_projections_max: usize) -> Self {
        let stride = device.limits().min_uniform_buffer_offset_alignment as u64;
        let uniform_buffer_size = std::mem::size_of::<UniformBuffer>() as u64;
        let size =
            (((uniform_buffer_size + stride - 1) / stride) * stride) * view_projections_max as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(uniform_buffer_size),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(uniform_buffer_size),
                }),
            }],
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }
}
