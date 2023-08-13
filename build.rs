fn main() {
    glib_build_tools::compile_resources(
        &["src/templates/"],
        "src/templates/resources.gresource.xml",
        "src.templates.gresource",
    );
}
