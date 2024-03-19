use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem};
use swc_core::ecma::{
    ast::{ImportDecl, Program, Str},
    transforms::testing::test,
    visit::{as_folder, FoldWith, VisitMut},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

pub struct TransformVisitor {
    pub specifiers: Vec<ImportSpecifier>,
}

impl TransformVisitor {
    pub fn new() -> Self {
        Self { specifiers: vec![] }
    }
}

impl VisitMut for TransformVisitor {
    fn visit_mut_module_items(&mut self, nodes: &mut Vec<ModuleItem>) {
        let mut transformed_nodes: Vec<ModuleItem> = vec![];

        for module_node in &mut nodes.clone() {
            match module_node {
                ModuleItem::ModuleDecl(ref mut node) => match node {
                    ModuleDecl::Import(ref mut node) => {
                        let imports_to_replace = vec!["useQuery", "useMutation", "gql"];
                        let path = node.src.value.to_string();

                        if path.eq("@apollo/client") {
                            let mut final_specifiers = vec![];
                            for specifier in &mut node.specifiers {
                                match specifier {
                                    ImportSpecifier::Named(spec) => {
                                        let mut import_name = spec.local.sym.to_string().clone();
                                        match &spec.imported {
                                            Some(import_export_name) => match import_export_name {
                                                ModuleExportName::Ident(ident) => {
                                                    import_name = ident.clone().sym.to_string()
                                                }
                                                ModuleExportName::Str(_) => {}
                                            },
                                            None => {}
                                        };
                                        if imports_to_replace.contains(&import_name.as_str()) {
                                            self.specifiers
                                                .push(ImportSpecifier::Named(spec.clone()));
                                            continue;
                                        }
                                        final_specifiers.push(ImportSpecifier::Named(spec.clone()));
                                        continue;
                                    }

                                    ImportSpecifier::Default(spec) => {
                                        final_specifiers
                                            .push(ImportSpecifier::Default(spec.clone()));
                                        continue;
                                    }
                                    ImportSpecifier::Namespace(spec) => {
                                        final_specifiers
                                            .push(ImportSpecifier::Namespace(spec.clone()));
                                        continue;
                                    }
                                }
                            }

                            node.specifiers = final_specifiers;
                        }

                        if node.specifiers.len() > 0 {
                            transformed_nodes
                                .push(ModuleItem::ModuleDecl(ModuleDecl::Import(node.clone())))
                        }
                    }
                    rest => transformed_nodes.push(ModuleItem::ModuleDecl(rest.clone())),
                },
                rest => transformed_nodes.push(rest.clone()),
            }
        }

        if self.specifiers.len() > 0 {
            transformed_nodes.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                span: DUMMY_SP,
                specifiers: self.specifiers.clone(),
                src: Box::new(Str::from("@repo/ui/gql")),
                type_only: false,
                with: None,
            })))
        }

        *nodes = transformed_nodes.clone();
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor::new()))
}

// An example to test plugin transform.
// Recommended strategy to test plugin's transform is verify
// the Visitor's behavior, instead of trying to run `process_transform` with mocks
// unless explicitly required to do so.
test!(
    Default::default(),
    |_| as_folder(TransformVisitor::new()),
    boo,
    // Input codes
    r#"import { useQuery, gql } from "@apollo/client""# // Output codes after transformed with plugin
);
