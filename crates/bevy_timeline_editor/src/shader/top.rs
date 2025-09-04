use bevy::asset::Asset;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, StandardMaterial};
use bevy::prelude::TypePath;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{AsBindGroup, CompareFunction, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError};

pub type AlwaysTopMaterial = ExtendedMaterial<StandardMaterial, AlwaysOnTopExt>;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct AlwaysOnTopExt {
    _name:()
}

impl MaterialExtension for AlwaysOnTopExt {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> bevy::prelude::Result<(), SpecializedMeshPipelineError> {
        if let Some(ds) = &mut descriptor.depth_stencil {
            ds.depth_compare = CompareFunction::Always; // 깊이 테스트 무조건 통과
            ds.depth_write_enabled = false;             // 깊이 버퍼에 기록 안 함
        }
        Ok(())
    }
}