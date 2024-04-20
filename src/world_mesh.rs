use bevy::{
    core_pipeline::core_2d::Transparent2d,
    pbr::MeshFlags,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PushConstantRange,
            RenderPipelineDescriptor, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexBufferLayout, VertexFormat,
            VertexState, VertexStepMode,
        },
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget, VisibleEntities},
        Extract, Render, RenderApp, RenderSet,
    },
    sprite::{
        extract_mesh2d, DrawMesh2d, Material2dBindGroupId, Mesh2dHandle, Mesh2dPipeline,
        Mesh2dPipelineKey, Mesh2dTransforms, RenderMesh2dInstance, RenderMesh2dInstances,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
    utils::FloatOrd,
};

/// A marker component for world meshes
#[derive(Component, Default)]
pub struct WorldMesh2d;

/// Custom pipeline for world meshes
#[derive(Resource)]
pub struct WorldMesh2dPipeline {
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for WorldMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`
impl SpecializedRenderPipeline for WorldMesh2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        let formats = vec![
            // Position
            VertexFormat::Float32x3,
            // Color
            VertexFormat::Uint32,
            // Local Position
            VertexFormat::Float32x2,
            // Neighbors
            VertexFormat::Uint32,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        let format = match key.contains(Mesh2dPipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        // I have no idea what this means
        let mut push_constant_ranges = Vec::with_capacity(1);
        if cfg!(all(
            feature = "webgl2",
            target_arch = "wasm32",
            not(feature = "webgpu")
        )) {
            push_constant_ranges.push(PushConstantRange {
                stages: ShaderStages::VERTEX,
                range: 0..4,
            });
        }

        RenderPipelineDescriptor {
            vertex: VertexState {
                // Use our custom shader
                shader: WORLD_MESH_SHADER_HANDLE,
                entry_point: "vertex".into(),
                shader_defs: vec![],
                // Use our custom vertex buffer
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                // Use our custom shader
                shader: WORLD_MESH_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform
                self.mesh2d_pipeline.mesh_layout.clone(),
            ],
            push_constant_ranges,
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("world_mesh2d_pipeline".into()),
        }
    }
}

// This specifies how to render a world mesh
type DrawWorlddMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Draw the mesh
    DrawMesh2d,
);

// The custom shader can be inline like here, included from another file at build time
// using `include_str!()`, or loaded like any other asset with `asset_server.load()`.
const WORLD_MESH_SHADER: &str = include_str!("../assets/shader/world_mesh_2d.wgsl");

/// Plugin that renders [`WorldMesh2d`]s
pub struct WorldMeshPlugin;

/// Handle to the custom shader with a unique random ID
pub const WORLD_MESH_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(13828845428412094821);

impl Plugin for WorldMeshPlugin {
    fn build(&self, app: &mut App) {
        // Load our custom shader
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.insert(
            WORLD_MESH_SHADER_HANDLE,
            Shader::from_wgsl(WORLD_MESH_SHADER, file!()),
        );

        // Register our custom draw function, and add our render systems
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_command::<Transparent2d, DrawWorlddMesh2d>()
            .init_resource::<SpecializedRenderPipelines<WorldMesh2dPipeline>>()
            .add_systems(ExtractSchedule, extract_world_mesh2d.after(extract_mesh2d))
            .add_systems(Render, queue_world_mesh2d.in_set(RenderSet::QueueMeshes));
    }

    fn finish(&self, app: &mut App) {
        // Register our custom pipeline
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<WorldMesh2dPipeline>();
    }
}

pub fn extract_world_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    // When extracting, you must use `Extract` to mark the `SystemParam`s
    // which should be taken from the main world.
    query: Extract<
        Query<(Entity, &ViewVisibility, &GlobalTransform, &Mesh2dHandle), With<WorldMesh2d>>,
    >,
    mut render_mesh_instances: ResMut<RenderMesh2dInstances>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, view_visibility, transform, handle) in &query {
        if !view_visibility.get() {
            continue;
        }

        let transforms = Mesh2dTransforms {
            transform: (&transform.affine()).into(),
            flags: MeshFlags::empty().bits(),
        };

        values.push((entity, WorldMesh2d));
        render_mesh_instances.insert(
            entity,
            RenderMesh2dInstance {
                mesh_asset_id: handle.0.id(),
                transforms,
                material_bind_group_id: Material2dBindGroupId::default(),
                automatic_batching: false,
            },
        );
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

#[allow(clippy::too_many_arguments)]
pub fn queue_world_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    world_mesh2d_pipeline: Res<WorldMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<WorldMesh2dPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_mesh_instances: Res<RenderMesh2dInstances>,
    mut views: Query<(
        &VisibleEntities,
        &mut RenderPhase<Transparent2d>,
        &ExtractedView,
    )>,
) {
    if render_mesh_instances.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase, view) in &mut views {
        let draw_world_mesh2d = transparent_draw_functions.read().id::<DrawWorlddMesh2d>();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Some(mesh_instance) = render_mesh_instances.get(visible_entity) {
                let mesh2d_handle = mesh_instance.mesh_asset_id;
                let mesh2d_transforms = &mesh_instance.transforms;
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(mesh2d_handle) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&pipeline_cache, &world_mesh2d_pipeline, mesh2d_key);

                let mesh_z = mesh2d_transforms.transform.translation.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_world_mesh2d,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: 0..1,
                    dynamic_offset: None,
                });
            }
        }
    }
}
