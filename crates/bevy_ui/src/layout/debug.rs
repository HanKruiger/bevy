use core::fmt::Write;

use taffy::{NodeId, TraversePartialTree};

use bevy_ecs::prelude::Entity;
use bevy_platform::collections::HashMap;

use crate::layout::ui_surface::UiSurface;

/// Prints a debug representation of the computed layout of the UI layout tree for each window.
pub fn print_ui_layout_tree(ui_surface: &UiSurface) {
    let taffy_to_entity: HashMap<NodeId, Entity> = ui_surface
        .entity_to_taffy
        .iter()
        .map(|(entity, node)| (node.id, *entity))
        .collect();
    for (&entity, &viewport_node) in &ui_surface.root_entity_to_viewport_node {
        let mut out = String::new();
        print_node(
            ui_surface,
            &taffy_to_entity,
            entity,
            viewport_node,
            false,
            String::new(),
            &mut out,
        );

        tracing::info!("Layout tree for camera entity: {entity}\n{out}");
    }
}

/// Recursively navigates the layout tree printing each node's information.
fn print_node(
    ui_surface: &UiSurface,
    taffy_to_entity: &HashMap<NodeId, Entity>,
    entity: Entity,
    node: NodeId,
    has_sibling: bool,
    lines_string: String,
    acc: &mut String,
) {
    let tree = &ui_surface.taffy;
    let layout = tree.layout(node).unwrap();
    let style = tree.style(node).unwrap();

    let num_children = tree.child_count(node);

    let display_variant = match (num_children, style.display) {
        (_, taffy::style::Display::None) => "NONE",
        (0, _) => "LEAF",
        (_, taffy::style::Display::Flex) => "FLEX",
        (_, taffy::style::Display::Grid) => "GRID",
        (_, taffy::style::Display::Block) => "BLOCK",
    };

    let fork_string = if has_sibling {
        "├── "
    } else {
        "└── "
    };
    writeln!(
        acc,
        "{lines}{fork} {display} [x: {x:<4} y: {y:<4} width: {width:<4} height: {height:<4}] ({entity}) {measured}",
        lines = lines_string,
        fork = fork_string,
        display = display_variant,
        x = layout.location.x,
        y = layout.location.y,
        width = layout.size.width,
        height = layout.size.height,
        measured = if tree.get_node_context(node).is_some() { "measured" } else { "" }
    ).ok();
    let bar = if has_sibling { "│   " } else { "    " };
    let new_string = lines_string + bar;

    // Recurse into children
    for (index, child_node) in tree.children(node).unwrap().iter().enumerate() {
        let has_sibling = index < num_children - 1;
        let child_entity = taffy_to_entity.get(child_node).unwrap();
        print_node(
            ui_surface,
            taffy_to_entity,
            *child_entity,
            *child_node,
            has_sibling,
            new_string.clone(),
            acc,
        );
    }
}
