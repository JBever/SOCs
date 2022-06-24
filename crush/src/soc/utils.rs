//! Module providing a set of tools to create `System` of bdds from file,
//! print a Bdd to .dot format for visualization, print systems to .bdd format
//! and needed structures for it.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::Child;
use std::str::FromStr;

use nom::digit;
use nom::types::CompleteStr;

use crate::soc::{
    bdd::Bdd,
    Id,
    system::System};

/// A specification of a `Node` inside a Bdd
#[derive(Debug,Clone)]
pub struct NodeSpec {
    id:Id,
    e0:Id,
    e1:Id
}

impl NodeSpec {
    /// Create a new `NodeSpec`
    pub fn new(id:Id,e0:Id,e1:Id) -> NodeSpec {
        NodeSpec{
            id,
            e0,
            e1
        }
    }

    /// Swap the value of `e0` and `e1`, flipping the edges of the `NodeSpec`
    pub fn flip_edge(&mut self){
        let e0_save = self.e0;
        self.e0 = self.e1;
        self.e1 = e0_save;
    }
}

/// A specification of a `Level` inside a Bdd.
/// 
/// `lhs` contains the left hand side equations of the level defined as
/// a `Vec` of `i64`. We use `i64` to allow for `-1` value which we will
/// later remove when creating the `System`. A vec![1,2,4] as `lhs` means
/// the equations is x1 + x2 + x4.
#[derive(Debug,Clone)]
pub struct LevelSpec {
    lhs:Vec<i64>,
    rhs:Vec<NodeSpec>
}

impl LevelSpec {
    /// Return a new `LevelSpec`
    pub fn new(lhs: Vec<i64>, rhs: Vec<NodeSpec>) -> LevelSpec{
        LevelSpec{
            lhs,
            rhs
        }
    }

    /// Remove each value in `lhs` equal to `-1`. If the number
    /// of -1 removed is 1 mod 2, then flip the edges of all the `NodeSpec`
    /// in `rhs`.
    pub fn remove_minus_one(&mut self) {
        let mut n = 0;
        self.lhs.retain(|i| {
            if i == &-1 {
                n += 1;
                return false
            }
            true
        });
        if n%2 != 0 {
            self.flip_nodes_edges();
        }
    }

    /// Calls the function `flip_edge` on all the nodes of `rhs`
    pub fn flip_nodes_edges(&mut self) {
        self.rhs.iter_mut().map(|node| node.flip_edge()).collect()
    }

}

/// A specification of Bdd
#[derive(Debug,Clone)]
pub struct BddSpec {
    id: Id,
    levels:Vec<LevelSpec>,
}

impl BddSpec {
    /// Return a new `BddSpec`
     pub fn new(id: Id, levels: Vec<LevelSpec>) -> BddSpec {
         BddSpec{
             id,
             levels
         }
     }
}

/// A specification of a system of Bdd
#[derive(Debug,Clone)]
pub struct SystemSpec {
    nvar:usize,
    bdds:Vec<BddSpec>,
}

impl SystemSpec {
    /// Return a new `SystemSpec`
    pub fn new(nvar:usize, bdds: Vec<BddSpec>) -> SystemSpec {
        SystemSpec{
            nvar,
            bdds
        }
    }
}

/// From a `SystemSpec` build a `System` following the specifications.
/// 
/// We create an empty `System` with the `nvar` set to the spec and 
/// push to it every `Bdd` created using the spec.
/// If some Id of Bdds in the spec are not unique their order is used as Id
pub fn build_system_from_spec(mut spec: SystemSpec) -> System {
    let mut system = System::new();
    system.set_nvar(spec.nvar as usize);
    let ids:HashSet<Id> = spec.bdds.iter().map(|bdd| bdd.id).collect();
    let nbr_bdd = spec.bdds.len();
    for (i,bdd_spec) in spec.bdds.iter_mut().enumerate(){
         if ids.len() != nbr_bdd {
            bdd_spec.id = Id::new(i);
        }
        system.push_bdd(build_bdd_from_spec(bdd_spec,spec.nvar as usize)).expect("No reason to crash since we are using the nvar of the system
        to set the one of the Bdds we are pushing");
    }
    system
}

/// From a `BddSpec` and a `nvar` build a `Bdd` following the specifications.
/// 
/// We create an empty `Bdd`, set its `id` according to the spec then create all the levels
/// (removing the `-1` from the `lhs` beforehand) without connecting the nodes.
/// 
/// Once all the level have been created we connect all the nodes to each other following the
/// `e0` and `e1` specs. All the id of the nodes are then reset to initialize `next_id` of the
/// `Bdd`. Finally we remove any jumping edges by calling `add_same_edge_node_at_level` on all the
/// levels of the `Bdd`.
/// WARNING! There is an unconfirmed case which indicates that the removal of jumping edges does NOT
/// work as intended! This will be investigated when I get the time.
// FIXME, the case referred to is the original PRINCE or LowMC S-box used in our differential
// experiments. It was built from a .bdd file, and we did had to change the .bdd to not include
// jumping edges b/c they caused us trouble. I was not aware of the fact that this fn is supposed
// to handle jumping edges at the time, otherwise I would have looked into it then. Maybe it's a
// too early short-circuit again?
// Of course, it may also have been some other error on our part, which is why this case is
// "unconfirmed".
pub fn build_bdd_from_spec(spec: &mut BddSpec, nvar: usize) -> Bdd {
    let mut bdd = Bdd::new();
    bdd.set_id(spec.id);
    let next_id = spec.levels.iter().fold(0,|last_id,level|
    {
        let level_id =level.rhs.iter().fold(0,|last_id_level,node| {
            if *node.id > last_id_level {
                *node.id
            } else {
                last_id_level
            }
        });
        if level_id>last_id {
            level_id
        } else {
            last_id
        }
    });
    for (i,level_spec) in spec.levels.iter_mut().enumerate(){
        level_spec.remove_minus_one();
        bdd.add_level();
        bdd.set_lhs_level(i,level_spec.lhs.iter().map(|i| *i as usize).collect(),nvar);
        bdd.add_nodes_to_level(i,level_spec.rhs.iter().map(|node| node.id).collect());
    }
    bdd.set_next_id(next_id+1);
    for level_spec in spec.levels.iter(){
        for node_spec in level_spec.rhs.iter(){
            if *node_spec.e0 != 0 {
                bdd.connect_nodes_from_spec(node_spec.id, node_spec.e0, 0);
            }
            if *node_spec.e1 != 0 {
                bdd.connect_nodes_from_spec(node_spec.id, node_spec.e1, 1);
            }
        }
    }
    if spec.levels.len() > 2 {
        for i in 1..spec.levels.len()-2 {
            bdd.add_same_edges_node_at_level(i);
        }
    }
    bdd
}


named!(i64 <CompleteStr, i64>,
ws!(
    map_res!(digit,|CompleteStr(s)| FromStr::from_str(s))
));

named!(usize <CompleteStr, usize>,
ws!(
    map_res!(digit,|CompleteStr(s)| FromStr::from_str(s))
));

named!(line_break <CompleteStr,Option<CompleteStr>>,
    opt!(alt!(tag!("\n")|tag!("\r\n")))
);

named!(minus_one <CompleteStr, i64>,
ws!(
    map_res!(
        recognize!(
            do_parse!(
                opt!(tag!("-")) >>
                digit >>
                ()
            )
        ),
    |CompleteStr(s)| FromStr::from_str(s))
));

named!(parameters<CompleteStr, (usize,usize)>,
    do_parse!(
        a: usize >>
        b: usize >>
        (a,b)
));

named!(var<CompleteStr,i64>,
    do_parse!(   
        opt!(alt!(char!('+')))>>
        a: alt!(i64 | minus_one)>>
        (a)
));

//pub because we use it in the bdd! macro
named!(pub vars<CompleteStr, Vec<i64>>,
    many0!(
        var
));

named!(lhs<CompleteStr,Vec<i64>>,
    do_parse!(
        a:vars>>
        (a)
));

named!(node<CompleteStr,NodeSpec>,
    do_parse!(
        char!('(')>>
        id: usize >>
        char!(';')>>
        e0: usize >>
        char!(',')>>
        e1: usize >>
        char!(')')>>
        (NodeSpec::new(Id::new(id), Id::new(e0), Id::new(e1)))
    )
);

named!(rhs<CompleteStr,Vec<NodeSpec>>,
    many0!(
        node
));

named!(level<CompleteStr,LevelSpec>,
    do_parse!(
        a:lhs>>
        char!(':')>>
        b:rhs>>
        char!('|')>>
        line_break>>
        (LevelSpec::new(a, b))
));

named!(levels<CompleteStr,Vec<LevelSpec>>,
    many0!(
        level
));

named!(bdd<CompleteStr,BddSpec>,
    do_parse!(
        param: parameters>>
        line_break>>
        levels: levels>>
        tag!("---")>>
        line_break>>
        (BddSpec::new(Id::new(param.0 as usize), levels))
));

named!(bdds<CompleteStr,Vec<BddSpec>>,
    many0!(
        bdd
));

named!(full_parser<CompleteStr,SystemSpec>,
    do_parse!(
        params:parameters>>
        line_break>>
        bdds:bdds>>
        (SystemSpec::new(params.0,bdds))
    )
);

/// Return a SystemSpec from the parsing of a .bdd file using the correct format
pub fn parse_system_spec_from_file(path: &PathBuf) -> SystemSpec {
    let file = File::open(path).unwrap();
    let mut file_content = String::new();
    BufReader::new(file).read_to_string(&mut file_content).unwrap();
    let result = full_parser(CompleteStr(&file_content)).expect("Parsing file");
    result.1
}

/// Write `.dot` language representation of the given bdd to a file at path
pub fn print_bdd_to_dot_format(bdd: &Bdd, path:&PathBuf) {
    let write_file = File::create(path).unwrap();
    let mut writer = BufWriter::new(&write_file);

    to_dot_format(&bdd, &mut writer);

    writer.flush().expect("Failed to write to file");
}

/// Write .bdd representation of a bdd to a Buffered write of a file
fn print_bdd_to_file_format(bdd: &Bdd,writer: &mut BufWriter<&File>){
    writeln!(writer, "{} {}",*bdd.get_id(),bdd.iter_levels().count()).unwrap();
    for level in bdd.iter_levels() {
        for (i,bit) in level.iter_set_lhs().enumerate(){
            if i != 0 {
                write!(writer,"+").unwrap();
            }
            write!(writer,"{}",bit).unwrap();
        }
        write!(writer,":").unwrap();
        for (id,node) in level.iter_nodes() {
            let e0 = match node.get_e0(){
                Some(e0) => *e0,
                None => 0,
            };
            let e1 = match node.get_e1(){
                Some(e1) => *e1,
                None => 0,
            };
            write!(writer,"({};{},{})",*id,e0,e1).unwrap();
        }
        writeln!(writer,"|").unwrap();
    }
    writeln!(writer,"---").unwrap();
}

/// Write .bdd representation of a system to a file at path
pub fn print_system_to_file(system: &System, path: &PathBuf){
    let write_file = File::create(path).unwrap();
    let mut writer = BufWriter::new(&write_file);
    writeln!(writer,"{} {}",system.get_nvar(),system.iter_bdds().len()).unwrap();
    let mut ids = Vec::new();
    for bdd in system.iter_bdds() {
        ids.push(bdd.0);
}
    ids.sort();
    for id in ids {
        print_bdd_to_file_format(&system.get_bdd(*id).unwrap().borrow(), &mut writer);
    }
}

/// Draw a graph representation of the Shard, using GraphViz.
/// The output format is PDF.
///
/// It is possible to use another function to instead output the dot-file of the shard. This allows
/// the user to draw using GraphViz as desired. This function is intended as a easy-to-use way
/// of generating snapshots of state. However, be mindful that GraphViz may use quite some time to
/// finish drawing the shard, even after this function returns the handle to the GraphViz process.
/// It is therefore *highly* recommended to always  use`.wait` on the handle to ensure that the
/// drawing process is complete, before exiting the main thread!
/// By returning the child handle, the caller is now free to decide when to wait for GraphViz to
/// finish drawing.
/// ---
/// **NOTE:** Requires that `GraphViz` is installed!
/// Tested on a Windows with Graphviz 3.0.0
///
/// **WARNING!** The resulting output file may be very large!
/// **WARNING** Failing to wait on the child process may lead to the failure of drawing the shard
/// to file.
/// **NOTE 1:** "Large" shards will take time to write to file. Patience is advised.
/// **NOTE 2:** When opening the pdf based on a "large" shard, it may initially appear empty.
/// When this is the case, it may be because it takes some time to load, or that you are viewing an
/// empty part of the drawing. Scrolling or zooming in/out may help.
/// ("Large" is hard to quantify, but my test file is only slightly more than 2mb large, yet took
/// many minutes for GraphViz to write to file. (Output size is about 6mb, GraphViz spent about
/// 30 min to draw...)).
pub fn draw_shard_as_pdf(shard: &Bdd, path:&PathBuf) -> Child {
    use std::process::{Command, Stdio};

    let mut args = vec!["-Tpdf",];
    let mut path = path.clone();
    path.set_extension("pdf");

    let out_path = format!("-o{}", path.as_os_str().to_str().unwrap());
    args.push(&out_path);


    let mut dot = Command::new("dot")
        .args(&args)
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to draw the shard to PDF.");

    {
        let child_in = dot.stdin.as_mut().expect("Failed to open child stdin");
        let mut writer = BufWriter::new(child_in);

        to_dot_format(&shard, &mut writer);
        writer.flush().unwrap();
    }
    dot
}

/// Write .dot language representation of the given shard into `writer`.
fn to_dot_format<W: Write> (shard: &Bdd, writer: &mut BufWriter<W>) {
    // Setup
    let num_levels = shard.iter_levels().count();

    // Metadata:
    writeln!(writer, "digraph \"DD\" {{").unwrap(); // I believe DD is just an ID.
    writeln!(writer, "center = true;").unwrap();
    writeln!(writer, "edge [dir = none];").unwrap(); // No arrowheads on the arrows

    // Writing the LHS of the graph
    writeln!(writer, "{{ node [shape = plaintext];").unwrap(); // No "bubble" around the algebraic expression
    writeln!(writer, "edge [style = invis];").unwrap(); // Draw no edges
    writeln!(writer, "\"CONST NODES\" [style = invis];").unwrap(); // End node? Invisible

    for (i,level) in shard.iter_levels().enumerate() {
        write!(writer, "\"{}. ",i).unwrap(); // Line/row number
        if level.iter_set_lhs().count() == 0 { // No variable is set
            write!(writer, "0").unwrap();
        } else {
            for (j, bit) in level.iter_set_lhs().enumerate() {
                if j > 0 {
                    write!(writer, " + ").unwrap();
                }
                write!(writer, "x{}", bit).unwrap();
            }
        }
        write!(writer, "\" -> ").unwrap();
        if i == num_levels - 2 { // Skip terminal lvl + started at index 0 ==> -2 ?
            break;
        }
    }
    writeln!(writer, "\"CONST NODES\";\n}}").unwrap();

    // Writing the RHS of the graph
    for (i,level) in shard.iter_levels().enumerate() {
        write!(writer, "{{ rank = same; ").unwrap(); // Tell GraphViz that these are on the same level
        write!(writer, "\"{}. ", i).unwrap(); // Line/row/"rank" number

        // I'm a bit unsure of the purpose of this if-else. I understand what it does, but not why.
        // Theory: Links these to the rank above w/same "ID"? Printed dot file both support and object
        // to this theory, and hard to find something in the GV doc.
        if level.iter_set_lhs().count() == 0 { // No variable is set
            write!(writer, "0").unwrap();
        } else {
            for (j,bit) in level.iter_set_lhs().enumerate() {
                if j > 0 {
                    write!(writer, " + ").unwrap();
                }
                write!(writer, "x{}", bit).unwrap();
            }
        }
        writeln!(writer, "\";").unwrap();

        // Add node to rank. (In GraphViz: level == rank)
        for (id,_) in level.iter_nodes(){
            // Remove the ID by setting label = "", and reducing drawing size by making the node shape to a point.
            writeln!(writer, "\"{}\" [label = \"\"; shape = point; width = 0.06];", *id).unwrap();
        }
        writeln!(writer, "}}").unwrap(); // Rank (/level) done

        if i == num_levels - 2 { // Skip terminal lvl + started at index 0 ==> -2 ?
            break;
        }
    }

    // Add terminal node, set node shape to box
    writeln!(writer, "{{ rank = same; \"CONST NODES\";").unwrap(); //
    writeln!(writer, "{{ node [shape = box]; \"{}\";", *shard.iter_levels().last().unwrap()
        .iter_nodes().last().unwrap()
        .0).unwrap();
    writeln!(writer, "}}").unwrap();
    writeln!(writer, "}}").unwrap();

    // Add edges between relevant nodes, including correct style
    for level in shard.iter_levels() {
        for (id,node) in level.iter_nodes() {
            if let Some(e0) = node.get_e0() {
                writeln!(writer, "\"{}\" -> \"{}\" [style = dashed];",*id,*e0).unwrap();
            }
            if let Some(e1) = node.get_e1() {
                writeln!(writer, "\"{}\" -> \"{}\";",*id,*e1).unwrap();
            }
        }
    }
    // Label the terminal node as the True node
    writeln!(writer, "\"{}\" [label = \"T\"];", *shard.iter_levels().last().unwrap()
        .iter_nodes().last().unwrap()
        .0).unwrap();
    writeln!(writer, "}}").unwrap();
}


