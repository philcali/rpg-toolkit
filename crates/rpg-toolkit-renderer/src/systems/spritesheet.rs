use bevy::prelude::*;

use crate::resources::RendererProjectData;

/// Builds a `TextureAtlasLayout` for a character spritesheet.
/// Layout: 3 columns (frames) × 4 rows (directions), each cell 24×32 pixels.
pub fn build_spritesheet_atlas() -> TextureAtlasLayout {
    TextureAtlasLayout::from_grid(UVec2::new(24, 32), 3, 4, None, None)
}

/// Startup system: loads spritesheet textures and atlas layouts into
/// `RendererProjectData` for all spritesheets in the project.
pub fn load_spritesheet_assets(
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut project_data: ResMut<RendererProjectData>,
) {
    let layout = build_spritesheet_atlas();
    let spritesheet_ids: Vec<_> = project_data.project_file.spritesheets.keys().cloned().collect();

    for ss_id in spritesheet_ids {
        let Some(ss) = project_data.project_file.spritesheets.get(&ss_id) else {
            continue;
        };
        let texture: Handle<Image> = asset_server.load(&ss.file_path);
        let atlas_handle = atlas_layouts.add(layout.clone());

        project_data.spritesheet_textures.insert(ss_id.clone(), texture);
        project_data.spritesheet_atlas_layouts.insert(ss_id, atlas_handle);
    }
}
