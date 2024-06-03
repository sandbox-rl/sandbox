use std::ops::{Deref, DerefMut};

use parking_lot::Mutex;
use windows::Win32::{
    Foundation::RECT,
    Graphics::{
        Direct3D::D3D_PRIMITIVE_TOPOLOGY,
        Direct3D11::{
            ID3D11BlendState, ID3D11Buffer, ID3D11ClassInstance, ID3D11DepthStencilState,
            ID3D11DeviceContext, ID3D11GeometryShader, ID3D11InputLayout, ID3D11PixelShader,
            ID3D11RasterizerState, ID3D11SamplerState, ID3D11ShaderResourceView,
            ID3D11VertexShader, D3D11_COMMONSHADER_CONSTANT_BUFFER_API_SLOT_COUNT,
            D3D11_COMMONSHADER_INPUT_RESOURCE_SLOT_COUNT, D3D11_COMMONSHADER_SAMPLER_SLOT_COUNT,
            D3D11_VIEWPORT, D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE,
        },
        Dxgi::Common::DXGI_FORMAT,
    },
};

#[derive(Default)]
pub struct BackupState(Mutex<InnerState>);
impl BackupState {
    pub fn save(&self, context: &ID3D11DeviceContext) {
        self.0.lock().save(context);
    }

    pub fn restore(&self, context: &ID3D11DeviceContext) {
        self.0.lock().restore(context);
    }
}

#[derive(Default)]
struct InnerState {
    scissor_rects: [RECT; D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE as _],
    scissor_count: u32,

    viewports: [D3D11_VIEWPORT; D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE as _],
    viewport_count: u32,

    raster_state: Option<ID3D11RasterizerState>,

    blend_state: Option<ID3D11BlendState>,
    blend_factor: [f32; 4],
    blend_mask: u32,

    depth_stencil_state: Option<ID3D11DepthStencilState>,
    stencil_ref: u32,

    pixel_shader_resources: Array<
        ID3D11ShaderResourceView,
        { (D3D11_COMMONSHADER_INPUT_RESOURCE_SLOT_COUNT - 1) as usize },
    >,
    samplers: Array<ID3D11SamplerState, { (D3D11_COMMONSHADER_SAMPLER_SLOT_COUNT - 1) as usize }>,

    vertex_shader: Option<ID3D11VertexShader>,
    vertex_shader_instances: Array<ID3D11ClassInstance, 256>,
    vertex_shader_instances_count: u32,

    geometry_shader: Option<ID3D11GeometryShader>,
    geometry_shader_instances: Array<ID3D11ClassInstance, 256>,
    geometry_shader_instances_count: u32,

    pixel_shader: Option<ID3D11PixelShader>,
    pixel_shader_instances: Array<ID3D11ClassInstance, 256>,
    pixel_shader_instances_count: u32,

    constant_buffers:
        Array<ID3D11Buffer, { (D3D11_COMMONSHADER_CONSTANT_BUFFER_API_SLOT_COUNT - 1) as usize }>,
    primitive_topology: D3D_PRIMITIVE_TOPOLOGY,

    index_buffer: Option<ID3D11Buffer>,
    index_buffer_format: DXGI_FORMAT,
    index_buffer_offset: u32,

    vertex_buffer: Option<ID3D11Buffer>,
    vertex_buffer_strides: u32,
    vertex_buffer_offsets: u32,

    input_layout: Option<ID3D11InputLayout>,
}

impl InnerState {
    pub fn save(&mut self, context: &ID3D11DeviceContext) {
        unsafe {
            context.RSGetScissorRects(
                &mut self.scissor_count,
                Some(self.scissor_rects.as_mut_ptr()),
            )
        };
        unsafe {
            context.RSGetViewports(&mut self.viewport_count, Some(self.viewports.as_mut_ptr()))
        };
        self.raster_state = unsafe { context.RSGetState().ok() };
        unsafe {
            context.OMGetBlendState(
                Some(&mut self.blend_state),
                Some(&mut self.blend_factor),
                Some(&mut self.blend_mask),
            )
        };
        unsafe {
            context.OMGetDepthStencilState(
                Some(&mut self.depth_stencil_state),
                Some(&mut self.stencil_ref),
            )
        };
        unsafe {
            context.PSGetShaderResources(0, Some(self.pixel_shader_resources.as_mut_slice()))
        };
        unsafe { context.PSGetSamplers(0, Some(self.samplers.as_mut_slice())) };

        self.pixel_shader_instances_count = 256;
        self.vertex_shader_instances_count = 256;
        self.geometry_shader_instances_count = 256;

        unsafe {
            context.PSGetShader(
                &mut self.pixel_shader,
                Some(self.pixel_shader_instances.as_mut_ptr()),
                Some(&mut self.pixel_shader_instances_count),
            )
        };

        unsafe {
            context.VSGetShader(
                &mut self.vertex_shader,
                Some(self.vertex_shader_instances.as_mut_ptr()),
                Some(&mut self.vertex_shader_instances_count),
            )
        };

        unsafe {
            context.GSGetShader(
                &mut self.geometry_shader,
                Some(self.geometry_shader_instances.as_mut_ptr()),
                Some(&mut self.geometry_shader_instances_count),
            )
        };

        unsafe { context.VSGetConstantBuffers(0, Some(self.constant_buffers.as_mut_slice())) };
        self.primitive_topology = unsafe { context.IAGetPrimitiveTopology() };
        unsafe {
            context.IAGetIndexBuffer(
                Some(&mut self.index_buffer),
                Some(&mut self.index_buffer_format),
                Some(&mut self.index_buffer_offset),
            )
        };
        unsafe {
            context.IAGetVertexBuffers(
                0,
                1,
                Some(&mut self.vertex_buffer),
                Some(&mut self.vertex_buffer_strides),
                Some(&mut self.vertex_buffer_offsets),
            )
        };

        self.input_layout = unsafe { context.IAGetInputLayout().ok() };
    }

    pub fn restore(&self, context: &ID3D11DeviceContext) {
        unsafe {
            context.RSSetScissorRects(Some(&self.scissor_rects[..self.scissor_count as usize]))
        };

        unsafe { context.RSSetViewports(Some(&self.viewports[..self.viewport_count as usize])) };

        unsafe { context.RSSetState(self.raster_state.as_ref()) };
        unsafe {
            context.OMSetBlendState(
                self.blend_state.as_ref(),
                Some(&self.blend_factor),
                self.blend_mask,
            )
        };
        unsafe {
            context.OMSetDepthStencilState(self.depth_stencil_state.as_ref(), self.stencil_ref)
        };
        unsafe { context.PSSetShaderResources(0, Some(self.pixel_shader_resources.as_slice())) };
        unsafe { context.PSSetSamplers(0, Some(self.samplers.as_slice())) };
        unsafe {
            context.PSSetShader(
                self.pixel_shader.as_ref(),
                Some(&self.pixel_shader_instances[..self.pixel_shader_instances_count as usize]),
            )
        };

        unsafe {
            context.VSSetShader(
                self.vertex_shader.as_ref(),
                Some(&self.vertex_shader_instances[..self.vertex_shader_instances_count as usize]),
            )
        };

        unsafe {
            context.GSSetShader(
                self.geometry_shader.as_ref(),
                Some(
                    &self.geometry_shader_instances
                        [..self.geometry_shader_instances_count as usize],
                ),
            )
        };

        unsafe { context.VSSetConstantBuffers(0, Some(self.constant_buffers.as_slice())) };
        unsafe { context.IASetPrimitiveTopology(self.primitive_topology) };
        unsafe {
            context.IASetIndexBuffer(
                self.index_buffer.as_ref(),
                self.index_buffer_format,
                self.index_buffer_offset,
            )
        };
        unsafe {
            context.IASetVertexBuffers(
                0,
                1,
                Some(&self.vertex_buffer),
                Some(&self.vertex_buffer_strides),
                Some(&self.vertex_buffer_offsets),
            )
        };

        unsafe { context.IASetInputLayout(self.input_layout.as_ref()) };
    }
}

#[repr(transparent)]
struct Array<T, const N: usize>([Option<T>; N]);

impl<T, const N: usize> Deref for Array<T, N> {
    type Target = [Option<T>; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for Array<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Array([const { None }; N])
    }
}
