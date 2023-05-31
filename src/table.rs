use std::collections::BTreeSet;

#[derive(Debug,Clone,Copy,Ord,PartialOrd,Eq,PartialEq)]
pub struct RowRef {
    table_idx: usize,
    row_idx: usize,
}

#[derive(Debug,Clone,Copy)]
pub struct TableRef {
    table_idx: usize,
}

#[derive(Debug)]
struct Row {
    styles: String,
    divs: Vec<String>,
    args: Option<Vec<usize>>,
}

#[derive(Debug)]
pub enum TableError {
    EmptySoft(String),
    FixedOnSoft(TableRef),
    SoftOnFixed(TableRef),
    MustBeOneSoftColumn(String),
    UnknownTable(TableRef),
    UnknownRow(RowRef),
    FixedRowTooLong {
        table: String,
        width: usize,
        pads: usize,
        asked: usize,
        unknown: usize,
    },
    SoftRowTooLong {
        table: String,
        min_width: usize,
    }
}

#[derive(Debug,Default)]
pub struct TableDrawer {
    row_set: BTreeSet<RowRef>
}

#[derive(Debug,Clone,Copy)]
enum TableType {
    Fixed(usize),
    Soft(usize),
}

#[derive(Debug)]
struct TableConf {
    index: usize,
    uid: String,
    tp: TableType,
    half_padding: usize,
    rows: Vec<Row>,
}

#[derive(Debug)]
pub struct TableBuilder {
    tables: Vec<TableConf>,
}
impl TableBuilder {
    pub fn new() -> TableBuilder {
        TableBuilder {
            tables: Vec::new(),
        }
    }
    pub fn table_fixed<S: ToString>(&mut self, uid: S, width: usize) -> TableRef {
        let res = TableRef { table_idx: self.tables.len() };
        self.tables.push(TableConf {
            index: res.table_idx,
            uid: uid.to_string(),
            tp: TableType::Fixed(width),
            half_padding: 2,
            rows: Vec::new(),
        });
        res
    }
    pub fn table_soft<S: ToString>(&mut self, uid: S, min_width: usize) -> TableRef {
        let res = TableRef { table_idx: self.tables.len() };
        self.tables.push(TableConf {
            index: res.table_idx,
            uid: uid.to_string(),
            tp: TableType::Soft(min_width),
            half_padding: 2,
            rows: Vec::new(),
        });
        res
    }
    pub fn with_half_padding(&mut self, table: TableRef, hp: usize) -> Result<(),TableError> {
        if self.tables.len() <= table.table_idx { return Err(TableError::UnknownTable(table)); }
        self.tables[table.table_idx].half_padding = hp;
        Ok(())
    }

    pub fn create_row_fixed(&mut self, table_ref: TableRef, columns: &[Option<usize>]) -> Result<RowRef,TableError> {
        if self.tables.len() <= table_ref.table_idx { return Err(TableError::UnknownTable(table_ref)); }
        let table_idx = table_ref.table_idx;
        
        match self.tables[table_idx].tp {
            TableType::Fixed(width) => create_fixed(&mut self.tables[table_idx],width,columns),
            TableType::Soft(_) => Err(TableError::FixedOnSoft(table_ref)),
        }
    }

    pub fn create_row_soft(&mut self, table_ref: TableRef, columns: Vec<SoftColumn>) -> Result<RowRef,TableError> {
        if self.tables.len() <= table_ref.table_idx { return Err(TableError::UnknownTable(table_ref)); }
        let table_idx = table_ref.table_idx;
        
        match self.tables[table_idx].tp {
            TableType::Fixed(_) => Err(TableError::SoftOnFixed(table_ref)),
            TableType::Soft(min_width) => create_soft(&mut self.tables[table_idx],min_width,columns),
        }
    }

    pub fn row(&self, row_ref: RowRef, class: &str, mut values: Vec<String>, drawer: &mut TableDrawer) -> Result<String,TableError> {
        if self.tables.len() <= row_ref.table_idx { return Err(TableError::UnknownRow(row_ref)); }
        if self.tables[row_ref.table_idx].rows.len() <= row_ref.row_idx { return Err(TableError::UnknownRow(row_ref)); }
        let row = &self.tables[row_ref.table_idx].rows[row_ref.row_idx];
        drawer.row_set.insert(row_ref);
        if row.divs.len() == 0 { return Ok(String::new()); }
        let cnt = row.divs.len() - 1;
        if cnt > values.len() {
            let i = values.len();
            for _ in  i .. cnt {
                values.push("&nbsp;".to_string());
            }
        }
        let mut res = format!("<div class='{}'>\n",class);
        for (i,d) in row.divs.iter().enumerate() {
            res += d;
            if i < values.len() {
                match &row.args {
                    None => res += &values[i],
                    Some(args) => if (i < args.len()) && (args[i] < values.len()) {
                        res += &values[args[i]];
                    },
                }
            }
        }
        res += "</div>";
        Ok(res)
    }

    pub fn styles(&self, drawer: &TableDrawer) -> String {
        let mut res = String::new();
        for rr in &drawer.row_set {
            if self.tables.len() <= rr.table_idx { continue; }
            if self.tables[rr.table_idx].rows.len() <= rr.row_idx { continue; }
            res += &self.tables[rr.table_idx].rows[rr.row_idx].styles;
        }
        res
    }
}


pub struct SoftColumn {
    pub percentage: Option<usize>,
    pub subcolumns: Vec<Option<usize>>,
}

#[derive(Debug)]
struct RealFixed {
    padding_left: usize,
    padding_right: usize,
    width: usize,
}
impl RealFixed {
    fn size(&self) -> usize {
        self.padding_left + self.padding_right + self.width
    }
}
#[derive(Debug)]
enum DivFixed {
    One(RealFixed),
    Vec(Vec<RealFixed>),
}
impl DivFixed {
    fn size(&self) -> usize {
        match self {
            DivFixed::One(d) => d.size(),
            DivFixed::Vec(v) => v.iter().fold(0,|acc,x| acc + x.size()),
        }
    }
    fn count(&self) -> usize {
        match self {
            DivFixed::One(_) => 1,
            DivFixed::Vec(v) => v.len(),
        }
    }
    fn divs(&self, styles: &mut String, pads: &mut Vec<String>, divs: &mut Vec<String>, args: &mut Vec<usize>, arg_offset: usize, arg_length: usize, cls_prefix: String, float: &str) {
        match self {
            DivFixed::One(RealFixed { padding_left, padding_right, width }) => {              
                *styles += &format!(".{} {{ padding-left: {}px; padding-right: {}px; width: {}px; float: {}; overflow: hidden; }}\n",cls_prefix,padding_left, padding_right, width, float);
                divs.push(format!("<div class='{}'>",cls_prefix));
                divs.push("</div>\n".to_string());
                pads.push(cls_prefix);
            },
            DivFixed::Vec(vf) => {
                let mut v = Vec::new();
                for (i,r) in vf.iter().enumerate() {
                    let RealFixed { padding_left, padding_right, width } = r;
                    let cls = format!("{}_c{}",cls_prefix,i);                    
                    *styles += &format!(".{} {{ padding-left: {}px; padding-right: {}px; width: {}px; overflow: hidden; float: left; }}\n",cls,padding_left, padding_right, width);
                    divs.push(format!("<div class='{}'>",cls));
                    divs.push("</div>\n".to_string());
                    pads.push(cls);
                }
                *styles += &format!(".{} {{ width: {}px; float: {}; overflow: hidden; }}\n",cls_prefix,vf.iter().fold(0,|acc,x| acc + x.size()),float);
                add_div(&cls_prefix,&mut v);
                divs.extend(v);
            },
        }
        for arg in 0 .. arg_length {
            args.push(arg_offset + arg);
        }
    }
}
#[derive(Debug)]
enum Fixed {
    None,
    Left(DivFixed),
    Right(DivFixed),
}
impl Fixed {
    fn right(right: Vec<usize>, most_right: bool, table: &mut TableConf) -> Fixed {
        Fixed::Right(match right.len() {
            1 => DivFixed::One(RealFixed {
                padding_left: table.half_padding,
                padding_right: match most_right { true => 2, false => 1 } * table.half_padding,
                width: right[0],
            }),
            _ => DivFixed::Vec({
                let mut v = Vec::new();
                let last = right.len() - 1;
                for (i,r) in right.into_iter().enumerate() {
                    v.push(RealFixed {
                        padding_left: table.half_padding,
                        padding_right: match (i == last) && most_right { true => 2, false => 1 } * table.half_padding,
                        width: r,
                    });
                }
                v
            }),
        })
    }
    fn left(left: Vec<usize>, most_left: bool, table: &mut TableConf) -> Fixed {
        Fixed::Left(match left.len() {
            1 => DivFixed::One(RealFixed {
                padding_left: match most_left { true => 2, false => 1 } * table.half_padding,
                padding_right: table.half_padding,
                width: left[0],
            }),
            _ => DivFixed::Vec({
                let mut v = Vec::new();
                for (i,r) in left.into_iter().enumerate() {
                    v.push(RealFixed {
                        padding_left: match (i == 0) && most_left{ true => 2, false => 1 } * table.half_padding,
                        padding_right: table.half_padding,
                        width: r,
                    });
                }
                v
            }),
        })
    }
    fn size(&self) -> usize {
        match self {
            Fixed::None => 0,
            Fixed::Left(d) |
            Fixed::Right(d) => d.size(),
        }
    }
    fn count(&self) -> usize {
        match self {
            Fixed::None => 0,
            Fixed::Left(d) |
            Fixed::Right(d) => d.count(),
        }
    }
}
#[derive(Debug)]
struct DivSoftReal {
    padding_left: usize,
    padding_right: usize,
    min_width: usize,
    margin: Option<usize>,
}
impl DivSoftReal {
    fn divs(&self, styles: &mut String, pads: &mut Vec<String>, divs: &mut Vec<String>, args: &mut Vec<usize>, arg_offset: usize, arg_length: usize, cls_prefix: String, float: &str) {  
        *styles += &format!(".{} {{ padding-left: {}px; padding-right: {}px; min-width: {}px; margin-{}: {}px; overflow: hidden; }}\n",cls_prefix,self.padding_left, self.padding_right, self.min_width, float, match self.margin { Some(m) => m, None => 0 });
        divs.push(format!("<div class='{}'>",cls_prefix));
        divs.push("</div>\n".to_string());
        pads.push(cls_prefix);
        for arg in 0 .. arg_length {
            args.push(arg_offset + arg);
        }
    }
}
#[derive(Debug)]
enum DivSoft {
    Phantom {
        min_width: usize,
        fixed: Fixed,
        soft: DivSoftReal,
        margin: Option<usize>,
    },
    Real(DivSoftReal),
}
impl DivSoft {
    fn count(&self) -> usize {
        match self {
            DivSoft::Phantom{ fixed, .. } => fixed.count() + 1,
            DivSoft::Real(..) => 1,
        }
    }
    fn divs(&self, styles: &mut String, pads: &mut Vec<String>, divs: &mut Vec<String>, args: &mut Vec<usize>, arg_offset: usize, arg_length: usize, cls_prefix: String, float: &str) {
        match &self {
            DivSoft::Phantom { fixed, soft, min_width, margin } => match fixed {
                Fixed::None => soft.divs(styles,pads,divs,args,arg_offset,arg_length,cls_prefix,float),
                Fixed::Left(left) => {
                    let l_cnt = left.count();
                    let r_cnt = 1; // soft.count();                    
                    let cls = format!("{}_l",cls_prefix);
                    let mut tmp_l = Vec::new();
                    left.divs(styles,pads,&mut tmp_l,args,arg_offset,l_cnt,cls,"left");                    
                    let cls = format!("{}_r",cls_prefix);
                    let mut tmp_r = Vec::new();
                    soft.divs(styles,pads,&mut tmp_r,args,arg_offset+l_cnt,r_cnt,cls,"left");
                    concat_divs(&mut tmp_l,tmp_r);
                    *styles += &format!(".{} {{ min-width: {}px; margin-{}: {}px; overflow: hidden; }}\n",cls_prefix,min_width, float, match margin { Some(m) => *m, None => 0 });
                    add_div("row",&mut tmp_l);
                    add_div(&cls_prefix,&mut tmp_l);
                    divs.extend(tmp_l);
                },
                Fixed::Right(right) => {
                    let r_cnt = right.count();
                    let l_cnt = 1; //soft.count();                    
                    let cls = format!("{}_r",cls_prefix);
                    let mut tmp_r = Vec::new();
                    right.divs(styles,pads,&mut tmp_r,args,arg_offset+l_cnt,r_cnt,cls,"right");
                    let cls = format!("{}_l",cls_prefix);
                    let mut tmp_l = Vec::new();
                    soft.divs(styles,pads,&mut tmp_l,args,arg_offset,l_cnt,cls,"right");
                    concat_divs(&mut tmp_r,tmp_l);
                    *styles += &format!(".{} {{ min-width: {}px; margin-{}: {}px; overflow: hidden; }}\n",cls_prefix,min_width, float, match margin { Some(m) => *m, None => 0 });
                    add_div("row",&mut tmp_r);
                    add_div(&cls_prefix,&mut tmp_r);
                    divs.extend(tmp_r);
                },
            },
            DivSoft::Real(r) => r.divs(styles,pads,divs,args,arg_offset,arg_length,cls_prefix,float),
        }
    }
}

#[derive(Debug)]
struct Phantom {
    //percentage: usize,
    //min_width: usize,
    fixed: Fixed,
    soft: DivSoft,
}
impl Phantom {
    fn count(&self) -> usize {
        self.fixed.count() + self.soft.count()
    }
    fn from_column(c: SoftColumn, min_width: usize, _percentage: usize, most_left: bool, most_right: bool, table: &mut TableConf) -> Result<Phantom,TableError> {
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut unk = 0;
        for col in c.subcolumns {
            match col {
                Some(w) if unk == 0 => left.push(w),
                Some(w) => right.push(w),
                None => unk += 1,
            }
        }
        if unk != 1 { return Err(TableError::MustBeOneSoftColumn(table.uid.clone())); }
        if left.len() == 0 {
            if right.len() == 0 {
                Ok(Phantom{
                    //percentage, min_width,
                    fixed: Fixed::None, soft: DivSoft::Real(DivSoftReal {
                        min_width: min_width - (match most_right { true => 2, false => 1 } + match most_left { true => 2, false => 1 }) * table.half_padding,
                        padding_left: match most_left { true => 2, false => 1 } * table.half_padding,
                        padding_right: match most_right { true => 2, false => 1 } * table.half_padding,
                        margin: None,
                    }),
                })
            } else {
                let fixed = Fixed::right(right,most_right,table);
                let margin_right = fixed.size();
                if min_width <= margin_right {
                    return Err(TableError::SoftRowTooLong { table: table.uid.clone(), min_width });
                }
                Ok(Phantom{
                    //percentage, min_width,
                    fixed,
                    soft: DivSoft::Real(DivSoftReal {
                        min_width: min_width - margin_right - match most_left { true => 3, false => 2 } * table.half_padding,
                        margin: Some(margin_right),
                        padding_left: match most_left { true => 2, false => 1 } * table.half_padding,
                        padding_right: table.half_padding,
                    }),                    
                })
            }
        } else {
            if right.len() == 0 {
                let fixed = Fixed::left(left,most_left,table);
                let margin_left = fixed.size();
                if min_width <= margin_left {
                    return Err(TableError::SoftRowTooLong { table: table.uid.clone(), min_width });
                }
                Ok(Phantom{
                    //percentage, min_width,
                    fixed,
                    soft: DivSoft::Real(DivSoftReal {
                        min_width: min_width - margin_left - match most_left { true => 3, false => 2 } * table.half_padding,
                        margin: Some(margin_left),
                        padding_left: table.half_padding,
                        padding_right: match most_left { true => 2, false => 1 } * table.half_padding,
                    }),                    
                })                    
            } else {
                let fixed_left = Fixed::left(left,most_left,table);
                let fixed_right = Fixed::right(right,most_right,table);
                let margin_left = fixed_left.size();
                let margin_right = fixed_right.size();
                if min_width <= (margin_right + margin_left) {
                    return Err(TableError::SoftRowTooLong { table: table.uid.clone(), min_width });
                }
                Ok(Phantom{
                    //percentage, min_width,
                    fixed: fixed_left,
                    soft: DivSoft::Phantom {
                        min_width: min_width - margin_left,
                        margin: Some(margin_left),
                        fixed: fixed_right,
                        soft: DivSoftReal {
                            min_width: min_width - margin_left - margin_right - table.half_padding * 2,
                            margin: Some(margin_right),
                            padding_left: table.half_padding,
                            padding_right: table.half_padding,
                        },                     
                    },
                })
            }
        }
    }
    fn divs(&self, styles: &mut String, pads: &mut Vec<String>, divs: &mut Vec<String>, args: &mut Vec<usize>, arg_offset: usize, arg_length: usize, cls_prefix: String) {
        // self.fixed self.soft
        match &self.fixed {
            Fixed::None => self.soft.divs(styles,pads,divs,args,arg_offset,arg_length,cls_prefix,"left"),
            Fixed::Left(left) => {
                let l_cnt = left.count();
                let r_cnt = self.soft.count();
                let cls = format!("{}_l",cls_prefix);
                let mut tmp_l = Vec::new();
                left.divs(styles,pads,&mut tmp_l,args,arg_offset,l_cnt,cls,"left");                
                let cls = format!("{}_r",cls_prefix);
                let mut tmp_r = Vec::new();
                self.soft.divs(styles,pads,&mut tmp_r,args,arg_offset+l_cnt,r_cnt,cls,"left");
                concat_divs(&mut tmp_l,tmp_r);
                divs.extend(tmp_l);
            },
            Fixed::Right(right) => {
                let r_cnt = right.count();
                let l_cnt = self.soft.count();                    
                let cls = format!("{}_r",cls_prefix);
                let mut tmp_r = Vec::new();
                right.divs(styles,pads,&mut tmp_r,args,arg_offset+l_cnt,r_cnt,cls,"right");
                let cls = format!("{}_l",cls_prefix);
                let mut tmp_l = Vec::new();
                self.soft.divs(styles,pads,&mut tmp_l,args,arg_offset,l_cnt,cls,"right");
                concat_divs(&mut tmp_r,tmp_l);
                divs.extend(tmp_r);
            },
        }
    }
}
#[derive(Debug)]
enum Div {
    One(Phantom),
    //Vec(Vec<Phantom>),
}

fn add_div(cls: &str, divs: &mut Vec<String>) {
    let ln = divs.len();
    if ln > 0 {
        divs[0] = format!("<div class='{}'>\n{}",cls,divs[0]);
        divs[ln-1] = format!("{}</div>\n",divs[ln-1]);
    }
}

fn concat_divs(l: &mut Vec<String>, mut r: Vec<String>) {
    match l.pop() {
        None => l.extend(r),
        Some(last_l) => {
            match r.len() > 0 { 
                true => r[0] = format!("{}{}",last_l,r[0]),
                false => r.push(last_l),
            }
            l.extend(r);
        },
    }
}

fn create_soft(table: &mut TableConf, min_width: usize, mut columns: Vec<SoftColumn>) -> Result<RowRef,TableError> {
    let div = match columns.len() {
        0 | 1 => match columns.pop() {
            None => return Err(TableError::EmptySoft(table.uid.clone())),
            Some(col) => Div::One(Phantom::from_column(col,min_width,100,true,true,table)?),
        },
        _ => {
            /*let mut v = Vec::new();
            for c in columns {
                v.push(Phantom::from_column(c,??,??)?);
            }
            Div::Vec(v)*/
            todo!()
        },
    };
    let row_idx = table.rows.len();
    let mut styles = String::new();
    let mut divs = Vec::new();
    let mut args = Vec::new();
    let mut pads = Vec::new();
    match div {
        Div::One(ph) => {
            let cnt = ph.count();
            ph.divs(&mut styles,&mut pads,&mut divs,&mut args,0,cnt,format!("{}",table.uid))
        },
    }
    let mut first = true;
    for cls in &pads {
        if !first { styles += ", "; }
        styles += ".";
        styles += cls;                        
        first = false;
    }
    styles += &format!("{{ padding-top: {}px; padding-bottom: {}px; }}\n", table.half_padding * 2, table.half_padding * 2);

    table.rows.push(Row{ styles, divs, args: Some(args) });
    Ok(RowRef{ table_idx: table.index, row_idx })
}

fn create_fixed(table: &mut TableConf, width: usize, columns: &[Option<usize>]) -> Result<RowRef,TableError> {
    let cnt = columns.len();
    let pads = (cnt + 1) * 2 * table.half_padding;
    let mut unk = 0;
    let mut asked = 0;
    for c in columns {
        match c {
            Some(w) => asked += w,
            None => unk += 1,
        }
    }
    if width >= (pads + asked + unk) {
        let row_idx = table.rows.len();
        let nw = width - pads - asked;
        let uw = nw / unk;
        let rw = nw % unk;
        
        let mut ctrl = 0;
        let mut u_idx = 0;
        let mut col_classes = Vec::new();
        let mut st = String::new();
        for (col_idx, c) in columns.iter().enumerate() {
            let w = match c {
                Some(w) => *w,
                None => {
                    let w = uw + match u_idx < rw { true => 1, false => 0 };
                    u_idx += 1;
                    w
                }
            };
            ctrl += w;
            let col_class = format!("{}_r{}_c{}",table.uid,row_idx,col_idx);
            st += &format!(".{} {{ width: {}px; }}\n",col_class,w);
            col_classes.push(col_class);
        }
        if (ctrl + pads) != width {
            return Err(TableError::FixedRowTooLong {
                table: table.uid.clone(), width, pads, asked, unknown: unk,
            });
        }
        
        let mut styles = String::new();
        
        let mut first = true;
        for cls in &col_classes {
            if !first { styles += ", "; }
            styles += ".";
            styles += cls;                        
            first = false;
        }
        styles += &format!("{{ padding: {}px; float: left; }}\n", table.half_padding * 2);
        
        let mut first = true;
        for cls in &col_classes[..(col_classes.len()-1)] {
            if !first { styles += ", "; }
            styles += ".";
            styles += cls;                        
            first = false;
        }
        styles += &format!("{{ padding-right: {}px; }}\n", table.half_padding);
        
        let mut first = true;
        for cls in &col_classes[1..] {
            if !first { styles += ", "; }
            styles += ".";
            styles += cls;                        
            first = false;
        }
        styles += &format!("{{ padding-left: {}px; }}\n", table.half_padding);
        styles += &st;
        
        let mut divs = col_classes.into_iter().enumerate().map(|(i,cls)| match i {
            0 => format!("<div class='{}'>",cls),
            _ => format!("</div>\n<div class='{}'>",cls),
        }).collect::<Vec<_>>();
        divs.push("</div>\n".to_string());
        table.rows.push(Row{ styles, divs, args: None });
        Ok(RowRef{ table_idx: table.index, row_idx })
    } else {
        Err(TableError::FixedRowTooLong {
            table: table.uid.clone(), width, pads, asked, unknown: unk,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_1() {
        let mut tb = TableBuilder::new();
        let table = tb.table_fixed("locs",250);
        let row = tb.create_row_fixed(table,&[None,Some(44),Some(44)]).unwrap();
        let mut drawer = TableDrawer::new();
        let body = tb.row(row,"locs_r0",vec!["Row1".to_owned(),"1".to_owned(),"10".to_owned()],&mut drawer).unwrap();
        let styles = tb.styles(&drawer);
        println!("{}",styles);
        println!("{}",body);
        panic!("");
    }

    #[test]
    fn soft_1() {
        let mut tb = TableBuilder::new();
        let table = tb.table_soft("pros",742);
        let row = tb.create_row_soft(table,vec![
            SoftColumn {
                percentage: None,
                subcolumns: vec![Some(150),None,Some(40)],
            },
        ]).unwrap();
        let mut drawer = TableDrawer::new();
        let body = tb.row(row,"pros_r0",vec!["Slot".to_owned(),"Data".to_owned(),"10".to_owned()],&mut drawer).unwrap();
        let styles = tb.styles(&drawer);
        println!("{}",styles);
        println!("{}",body);
        panic!("");
    }

     #[test]
    fn soft_2() {
        let mut tb = TableBuilder::new();
        let table = tb.table_soft("pros",742);
        let row = tb.create_row_soft(table,vec![
            SoftColumn {
                percentage: None,
                subcolumns: vec![None],
            },
        ]).unwrap();
        let mut drawer = TableDrawer::new();
        let body = tb.row(row,"pros_rh",vec!["Header".to_owned()],&mut drawer).unwrap();
        let styles = tb.styles(&drawer);
        println!("{}",styles);
        println!("{}",body);
        panic!("");
    }

    
}


