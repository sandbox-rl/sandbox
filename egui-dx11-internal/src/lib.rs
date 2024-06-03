#![feature(once_cell_try)]

use std::mem::{self};
use std::sync::OnceLock;

use backup::BackupState;
use clipboard::windows_clipboard::WindowsClipboardContext;
use clipboard::ClipboardProvider;
use egui::epaint::Primitive;
use egui::Context;
use input::InputCollector;
use mesh::{GpuMesh, GpuVertex};
use parking_lot::Mutex;
use shader::CompiledShaders;
use texture::TextureAllocator;
use windows::core::{s, w, Interface, HRESULT};
use windows::Win32::Foundation::{FALSE, HWND, LPARAM, LRESULT, RECT, TRUE, WPARAM};
use windows::Win32::Graphics::Direct3D::{
    D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0,
    D3D_FEATURE_LEVEL_11_1,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
    ID3D11RenderTargetView, ID3D11Texture2D, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_BLEND_DESC,
    D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD, D3D11_BLEND_SRC_ALPHA,
    D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_COMPARISON_ALWAYS, D3D11_CREATE_DEVICE_FLAG,
    D3D11_CULL_NONE, D3D11_FILL_SOLID, D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_INPUT_ELEMENT_DESC,
    D3D11_INPUT_PER_VERTEX_DATA, D3D11_RASTERIZER_DESC, D3D11_RENDER_TARGET_BLEND_DESC,
    D3D11_SAMPLER_DESC, D3D11_SDK_VERSION, D3D11_TEXTURE_ADDRESS_BORDER, D3D11_VIEWPORT,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32_UINT,
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED,
    DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    IDXGISwapChain, IDXGISwapChain_Vtbl, DXGI_SWAP_CHAIN_DESC,
    DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH, DXGI_SWAP_EFFECT_DISCARD,
    DXGI_USAGE_RENDER_TARGET_OUTPUT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, RegisterClassExW,
    UnregisterClassW, CS_HREDRAW, CS_VREDRAW, WNDCLASSEXW, WS_EX_RIGHTSCROLLBAR,
    WS_OVERLAPPEDWINDOW,
};

mod backup;
mod input;
mod mesh;
mod shader;
mod texture;

type WinResult<T> = windows::core::Result<T>;

pub struct Dx11App<T = ()> {
    dx: OnceLock<DxState>,
    ui: Mutex<Box<dyn FnMut(&Context, &mut T) + Send + 'static>>,
    ctx: Context,
    state: egui::mutex::Mutex<T>,
}

struct DxState {
    input_layout: ID3D11InputLayout,
    shaders: CompiledShaders,
    tex_alloc: TextureAllocator,
    backup: BackupState,
    render_view: Mutex<ID3D11RenderTargetView>,
    input_collector: Mutex<InputCollector>,
    hwnd: HWND,
}

impl DxState {
    const INPUT_ELEMENTS_DESC: [D3D11_INPUT_ELEMENT_DESC; 3] = [
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: s!("POSITION"),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 0,
            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: s!("TEXCOORD"),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: s!("COLOR"),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
    ];

    fn new(swapchain: &IDXGISwapChain) -> WinResult<Self> {
        let mut desc = DXGI_SWAP_CHAIN_DESC::default();
        let desc = unsafe { swapchain.GetDesc(&mut desc).map(|()| desc)? };

        let hwnd = desc.OutputWindow;

        let device: ID3D11Device = unsafe { swapchain.GetDevice() }?;

        let back_buffer: ID3D11Texture2D = unsafe { swapchain.GetBuffer(0) }?;

        let mut render_view = None;
        let render_view = unsafe {
            device
                .CreateRenderTargetView(&back_buffer, None, Some(&mut render_view))
                .map(|()| render_view.unwrap_unchecked())?
        };

        let shaders = CompiledShaders::new(&device)?;

        let mut input_layout = None;
        let input_layout = unsafe {
            device
                .CreateInputLayout(
                    &Self::INPUT_ELEMENTS_DESC,
                    &shaders.bytecode,
                    Some(&mut input_layout),
                )
                .map(|()| input_layout.unwrap_unchecked())?
        };

        Ok(DxState {
            backup: BackupState::default(),
            input_collector: Mutex::new(InputCollector::new(hwnd)),
            tex_alloc: TextureAllocator::default(),
            render_view: Mutex::new(render_view),
            input_layout,
            shaders,
            hwnd,
        })
    }

    fn get_screen_size(&self) -> (f32, f32) {
        let mut rect = RECT::default();
        _ = unsafe { GetClientRect(self.hwnd, &mut rect) };
        (
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        )
    }

    fn get_viewport(&self) -> D3D11_VIEWPORT {
        let (w, h) = self.get_screen_size();
        D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: w,
            Height: h,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        }
    }
}

impl<T> Dx11App<T> {
    pub fn new(ui: impl FnMut(&Context, &mut T) + Send + 'static, state: T) -> Self {
        Self {
            dx: OnceLock::new(),
            ui: Mutex::new(Box::new(ui)),
            ctx: Context::default(),
            state: egui::mutex::Mutex::new(state),
        }
    }

    fn set_blend_state(device: &ID3D11Device, context: &ID3D11DeviceContext) -> WinResult<()> {
        let targets: [D3D11_RENDER_TARGET_BLEND_DESC; 8] = [
            D3D11_RENDER_TARGET_BLEND_DESC {
                BlendEnable: TRUE,
                SrcBlend: D3D11_BLEND_SRC_ALPHA,
                DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
                BlendOp: D3D11_BLEND_OP_ADD,
                SrcBlendAlpha: D3D11_BLEND_ONE,
                DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
                BlendOpAlpha: D3D11_BLEND_OP_ADD,
                RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
            },
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        ];

        let blend_desc = D3D11_BLEND_DESC {
            AlphaToCoverageEnable: FALSE,
            IndependentBlendEnable: FALSE,
            RenderTarget: targets,
        };

        let mut blend_state = None;
        let blend_state = unsafe {
            device
                .CreateBlendState(&blend_desc, Some(&mut blend_state))
                .map(|()| blend_state.unwrap_unchecked())?
        };

        unsafe { context.OMSetBlendState(&blend_state, Some(&[0., 0., 0., 0.]), 0xffffffff) };

        Ok(())
    }

    fn set_raster_options(device: &ID3D11Device, context: &ID3D11DeviceContext) -> WinResult<()> {
        let raster_desc = D3D11_RASTERIZER_DESC {
            FillMode: D3D11_FILL_SOLID,
            CullMode: D3D11_CULL_NONE,
            FrontCounterClockwise: FALSE,
            DepthBias: 0,
            DepthBiasClamp: 0.0,
            SlopeScaledDepthBias: 0.0,
            DepthClipEnable: FALSE,
            ScissorEnable: TRUE,
            MultisampleEnable: FALSE,
            AntialiasedLineEnable: FALSE,
        };

        let mut options = None;
        let options = unsafe {
            device
                .CreateRasterizerState(&raster_desc, Some(&mut options))
                .map(|()| options.unwrap_unchecked())?
        };

        unsafe { context.RSSetState(&options) };

        Ok(())
    }

    fn set_sampler_state(device: &ID3D11Device, context: &ID3D11DeviceContext) -> WinResult<()> {
        let desc = D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D11_TEXTURE_ADDRESS_BORDER,
            AddressV: D3D11_TEXTURE_ADDRESS_BORDER,
            AddressW: D3D11_TEXTURE_ADDRESS_BORDER,
            MipLODBias: 0.0,
            ComparisonFunc: D3D11_COMPARISON_ALWAYS,
            MinLOD: 0.0,
            MaxLOD: 0.0,
            BorderColor: [1.0; 4],
            MaxAnisotropy: 0,
        };

        let mut sampler = None;
        let sampler = unsafe {
            device
                .CreateSamplerState(&desc, Some(&mut sampler))
                .map(|()| sampler.unwrap_unchecked())?
        };

        unsafe { context.PSSetSamplers(0, Some(&[Some(sampler)])) };

        Ok(())
    }

    pub fn present(&self, swapchain: &IDXGISwapChain) -> WinResult<()> {
        let dx @ DxState {
            tex_alloc,
            shaders,
            input_layout,
            backup,
            render_view,
            input_collector,
            ..
        } = self.dx.get_or_try_init(|| DxState::new(swapchain))?;

        let device: ID3D11Device = unsafe { swapchain.GetDevice()? };
        let context = unsafe { device.GetImmediateContext() }?;

        backup.save(&context);

        let input = input_collector.lock().collect_input();

        let ui = &mut *self.ui.lock();

        let output = self
            .ctx
            .run(input, |ctx| (ui)(ctx, &mut *self.state.lock()));

        if !output.textures_delta.is_empty() {
            tex_alloc.process_deltas(&device, &context, output.textures_delta)?;
        }

        if !output.platform_output.copied_text.is_empty() {
            let _ = WindowsClipboardContext.set_contents(output.platform_output.copied_text);
        }

        let screen = dx.get_screen_size();

        let primitives = self
            .ctx
            .tessellate(output.shapes, self.ctx.pixels_per_point())
            .into_iter()
            .filter_map(|prim| {
                if let Primitive::Mesh(mesh) = prim.primitive {
                    GpuMesh::from_mesh(screen, mesh, prim.clip_rect)
                } else {
                    unimplemented!("Paint callbacks are not supported")
                }
            });

        Self::set_blend_state(&device, &context)?;
        Self::set_raster_options(&device, &context)?;
        Self::set_sampler_state(&device, &context)?;

        unsafe { context.RSSetViewports(Some(&[dx.get_viewport()])) };
        unsafe { context.OMSetRenderTargets(Some(&[Some(render_view.lock().clone())]), None) };
        unsafe { context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST) };
        unsafe { context.IASetInputLayout(input_layout) };

        for mesh in primitives {
            let idx = mesh::create_index_buffer(&device, &mesh)?;
            let vtx = mesh::create_vertex_buffer(&device, &mesh)?;

            let texture = tex_alloc.get_by_id(mesh.texture_id);

            unsafe {
                context.RSSetScissorRects(Some(&[RECT {
                    left: mesh.clip.left() as _,
                    top: mesh.clip.top() as _,
                    right: mesh.clip.right() as _,
                    bottom: mesh.clip.bottom() as _,
                }]))
            };

            if texture.is_some() {
                unsafe { context.PSSetShaderResources(0, Some(&[texture])) };
            }

            unsafe {
                context.IASetVertexBuffers(
                    0,
                    1,
                    Some(&Some(vtx)),
                    Some(&(mem::size_of::<GpuVertex>() as u32)),
                    Some(&0),
                )
            };

            unsafe { context.IASetIndexBuffer(&idx, DXGI_FORMAT_R32_UINT, 0) };

            unsafe { context.VSSetShader(&shaders.vertex, Some(&[])) };
            unsafe { context.PSSetShader(&shaders.pixel, Some(&[])) };

            unsafe { context.DrawIndexed(mesh.indicies.len() as u32, 0, 0) };
        }

        backup.restore(&context);

        Ok(())
    }

    pub fn resize_buffers(
        &self,
        swapchain: &IDXGISwapChain,
        original: impl FnOnce() -> HRESULT,
    ) -> WinResult<HRESULT> {
        let result = original();

        if let Some(dx) = self.dx.get() {
            let backbuffer: ID3D11Texture2D = unsafe { swapchain.GetBuffer(0) }?;

            let device: ID3D11Device = unsafe { swapchain.GetDevice() }?;

            let mut render_view = None;
            let render_view = unsafe {
                device
                    .CreateRenderTargetView(&backbuffer, None, Some(&mut render_view))
                    .map(|()| render_view.unwrap_unchecked())
            }?;

            *dx.render_view.lock() = render_view;
        }

        Ok(result)
    }

    pub fn wnd_proc(&self, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
        if let Some(dx) = self.dx.get() {
            dx.input_collector.lock().process(umsg, wparam.0, lparam.0);
        }

        self.ctx.wants_pointer_input()
    }
}

pub fn create_dummy_swapchain() -> WinResult<IDXGISwapChain> {
    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    let windowclass = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hInstance: unsafe { GetModuleHandleW(None).unwrap().into() },
        lpszClassName: w!("dummy"),
        ..Default::default()
    };

    unsafe { RegisterClassExW(&windowclass) };

    let window = unsafe {
        CreateWindowExW(
            WS_EX_RIGHTSCROLLBAR,
            windowclass.lpszClassName,
            w!("dummy window"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            100,
            100,
            None,
            None,
            windowclass.hInstance,
            None,
        )
    };

    let desc = DXGI_SWAP_CHAIN_DESC {
        BufferDesc: DXGI_MODE_DESC {
            Height: 100,
            Width: 100,
            RefreshRate: DXGI_RATIONAL {
                Numerator: 60,
                Denominator: 1,
            },
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
            Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
        },
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 1,
        OutputWindow: window,
        Windowed: TRUE,
        SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
        Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH.0 as u32,
    };

    let feature_levels = [D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1];
    let mut swapchain = None;
    let mut device = None;
    let mut context = None;

    unsafe {
        D3D11CreateDeviceAndSwapChain(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_FLAG(0),
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&desc),
            Some(&mut swapchain),
            Some(&mut device),
            None,
            Some(&mut context),
        )?
    };

    let swapchain = swapchain.unwrap();

    let _ = unsafe { DestroyWindow(window) };
    let _ = unsafe { UnregisterClassW(windowclass.lpszClassName, windowclass.hInstance) };

    Ok(swapchain)
}

pub fn swapchain_vtable() -> WinResult<&'static IDXGISwapChain_Vtbl> {
    let swapchain = create_dummy_swapchain()?;
    let vtable: *const _ = swapchain.vtable();
    Ok(unsafe { &*vtable })
}
