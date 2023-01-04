use std::ffi::CStr;

use ash::vk;

use crate::io::file;

use super::structures::PipelineInfo;

pub fn create_pipeline(
    device: &ash::Device,
    shader_name: &str,
    extent: &vk::Extent2D,
    formats: &Vec<vk::SurfaceFormatKHR>,
) -> PipelineInfo {
    let vert_module =
        create_shader_pipeline(device, file::read_file(&format!("{}_v.spv", shader_name)));

    let frag_module =
        create_shader_pipeline(device, file::read_file(&format!("{}_f.spv", shader_name)));

    let vertex_pipeline_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(&CStr::from_bytes_with_nul("main\0".as_bytes()).expect("Failed to convert to cstr"))
        .build();

    let fragment_pipeline_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(&CStr::from_bytes_with_nul("main\0".as_bytes()).expect("Failed to convert to cstr"))
        .build();

    let shader_stages = [
        vertex_pipeline_shader_stage_create_info,
        fragment_pipeline_shader_stage_create_info,
    ];

    let shader_modules = [vert_module, frag_module];

    let pipeline_dynamic_state_create_info = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
        .build();

    let pipeline_vertex_input_state_create_info =
        vk::PipelineVertexInputStateCreateInfo::builder().build();

    let pipeline_input_assembly_state_create_info =
        vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(extent.width as f32)
        .height(extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0)
        .build();

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D::builder().x(0).y(0).build())
        .extent(extent.clone())
        .build();

    let pipeline_viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .viewports(&[viewport])
        .scissor_count(1)
        .scissors(&[scissor])
        .build();

    let pipeline_rasterization_state_create_info =
        vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.0)
            .depth_clamp_enable(false) // TODO: change to true after enabling the GPU feature
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();

    let pipeline_multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .build();

    let pipeline_color_blend_attachement_state = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .build();

    let pipeline_color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(&[pipeline_color_blend_attachement_state])
        .build();

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder().build();

    let pipeline_layout =
        unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None) }
            .expect("Failed to create pipeline layout");

    let attachment_description = vk::AttachmentDescription::builder()
        .format(formats[0].format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build();

    let attachment_reference = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let subpass_description = vk::SubpassDescription::builder()
        .color_attachments(&[attachment_reference])
        .build();

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .build();

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&[attachment_description])
        .subpasses(&[subpass_description])
        .dependencies(&[dependency])
        .build();

    let render_pass = unsafe { device.create_render_pass(&render_pass_info, None) }
        .expect("Failed to create render pass");

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&pipeline_vertex_input_state_create_info)
        .input_assembly_state(&pipeline_input_assembly_state_create_info)
        .viewport_state(&pipeline_viewport_state_create_info)
        .rasterization_state(&pipeline_rasterization_state_create_info)
        .multisample_state(&pipeline_multisample_state_create_info)
        .color_blend_state(&pipeline_color_blend_state_create_info)
        .dynamic_state(&pipeline_dynamic_state_create_info)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build();

    let pipeline = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
    }
    .expect("Failed to create pipeline");

    PipelineInfo {
        pipeline,
        pipeline_layout,
        render_pass,
        shader_modules,
        render_pass_info,
    }
}

pub fn create_shader_pipeline(device: &ash::Device, code: Vec<u8>) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
        .code(unsafe { code.align_to::<u32>().1 })
        .build();
    unsafe { device.create_shader_module(&shader_module_create_info, None) }
        .expect("Failed to create shader module")
}