//! What the game draws for Heihe, counted: meshes, triangles, lights and boards.
//! For reading frame times against (northeast --bench).
//!
//! cargo run -p sim --bin scene_report

fn main() {
    let (city, pal) = match sim::load_heihe() {
        Ok(found) => found,
        Err(why) => {
            eprintln!("Heihe would not load: {why}");
            std::process::exit(1);
        }
    };
    let scene = sim::scene::build(&city, &pal);
    let tris = |s: &sim::soup::Soup| s.triangles();
    let (mut solid, mut glass, mut lit, mut meshes, mut crowns) = (0, 0, 0, 0, 0);
    for b in &scene.blocks {
        solid += tris(&b.solid);
        glass += tris(&b.glass);
        lit += tris(&b.lit);
        meshes += 1 + usize::from(b.glass.triangles() > 0) + usize::from(b.lit.triangles() > 0);
        crowns += usize::from(!b.crown.is_empty());
    }
    println!("Chinese bank: {} buildings drawn one by one, {meshes} meshes, {crowns} crowns", scene.blocks.len());
    println!("  triangles: walls and roofs {solid}, dark windows {glass}, lit windows {lit}");
    println!("far bank (one mesh each): buildings {}, glass {}, lit {}", tris(&scene.buildings), tris(&scene.glass), tris(&scene.lit));
    println!(
        "ground {}, props {} ({} chunks @ 120 m), lamp heads {} ({} chunks)",
        tris(&scene.ground),
        tris(&scene.props),
        scene.props.chunks(120.0).len(),
        tris(&scene.lamp_glow),
        scene.lamp_glow.chunks(120.0).len(),
    );
    println!(
        "street lamps: {} heads on {} posts (point lights: one per post by default, one per head with NORTHEAST_LAMPS=heads)",
        scene.lamp_heads.len(),
        scene.lamp_posts.len()
    );
    println!("shop boards (one mesh, material and texture each): {}", scene.boards.len());
    let total = solid + glass + lit + tris(&scene.buildings) + tris(&scene.glass) + tris(&scene.lit) + tris(&scene.ground) + tris(&scene.props) + tris(&scene.lamp_glow) + 2 * scene.boards.len();
    println!("triangles in all: {total}");
}
