use std::rc::Rc;
use std::io::prelude::*;

const LCHD: usize = 0;
const RCHD: usize = 1;

#[derive(Clone, PartialEq)]
enum TreeNode {
    Null,
    Node{
        key: i32,
        size: i32,
        chd: [Rc<TreeNode>; 2]
    }
} // Option<Node>

impl TreeNode {
    fn key(&self) -> i32 {
        match self {
            TreeNode::Null => 0,
            TreeNode::Node{ key: k, .. } => *k
        } 
    }

    fn size(&self) -> i32 {
        match self {
            TreeNode::Null => 0,
            TreeNode::Node{ size: sz, .. } => *sz
        } 
    }

    fn child(&self, dir: usize) -> Self {
        match self {
            TreeNode::Null => TreeNode::Null,
            TreeNode::Node{ chd: child, .. } => (*child[dir]).clone()
        } 
    }

    fn new(key: i32, dir: usize, ichd: TreeNode, ochd: TreeNode) -> Self {
        TreeNode::Node {
            key: key,
            size: 1 + ichd.size() + ochd.size(),
            chd: if dir == LCHD {
                [Rc::new(ichd), Rc::new(ochd)]
            } else {
                [Rc::new(ochd), Rc::new(ichd)]
            }
        }
    }

    fn replace_chd(&self, dir: usize, new_chd: TreeNode) -> Self {
        match self {
            TreeNode::Null => TreeNode::Null,
            TreeNode::Node{key: k, chd: ch, ..} => 
                TreeNode::new(*k, dir, new_chd, (*ch[dir ^ 1]).clone())
        } 
    }
}

#[derive(Clone)]
struct Splay {
    // The splay root
    root: Rc<TreeNode>,
    // The reversed temporary tree
    temp: Rc<TreeNode>
}

impl Splay {
    fn zig(&self, dir: usize) -> Self {
        Splay {
            root: Rc::new(self.root.child(dir)),
            temp: Rc::new(self.temp.replace_chd(dir,
                self.root.replace_chd(dir, self.temp.child(dir))))
        }
    }

    fn zigzig(&self, dir: usize) -> Self {
        let middle = self.root.child(dir);
        Splay {
            root: Rc::new(middle.child(dir)),
            temp: Rc::new(self.temp.replace_chd(dir, 
                TreeNode::new(middle.key(), dir,
                    self.temp.child(dir),
                    self.root.replace_chd(dir, middle.child(dir ^ 1)))))
        }
    }

    fn finish(&self, dir: usize) -> Self {
        // The front node of the reversed temporary tree
        let mut head = self.temp.child(dir);
        // The new child of head after putting head back
        let mut child = self.root.child(1 ^ dir);
        while let TreeNode::Node{..} = head {
            child = head.replace_chd(dir, child);
            head = head.child(dir);
        }
        Splay {
            root: Rc::new(self.root.replace_chd(1 ^ dir, child)),
            temp: Rc::new(self.temp.replace_chd(dir, TreeNode::Null))
        }
    }

    fn splay(&self, pred: &mut dyn FnMut(&TreeNode) -> Option<usize>) -> Self {
        let mut ret = self.clone();
        while let Some(dir1) = pred(&*ret.root) {
            if let TreeNode::Null = ret.root.child(dir1) {
                break;
            }
            match pred(&ret.root.child(dir1)) {
                None => {
                    ret = ret.zig(dir1);
                    break;
                }
                Some(dir2) => {
                    if let TreeNode::Null = ret.root.child(dir1).child(dir2) {
                        ret = ret.zig(dir1);
                        break;
                    } else if dir1 == dir2 {
                        ret = ret.zigzig(dir1);
                    } else {
                        ret = ret.zig(dir1).zig(dir2);
                    }
                }
            }
        }

        ret.finish(LCHD).finish(RCHD)
    }

    fn select(&self, k: i32) -> Self {
        let mut kval = k;
        let mut pred = |node: &TreeNode| {
            let pos = node.child(LCHD).size();
            return if kval == pos {
                None
            } else if kval < pos {
                Some(LCHD)
            } else {
                kval -= pos + 1;
                Some(RCHD)
            }
        };
        self.splay(&mut pred)
    }

    fn lowerbound(&self, x: i32) -> Self {
        let ret = self.splay(&mut |node: &TreeNode| {
            if x <= node.key() {
                Some(LCHD)
            } else { 
                Some(RCHD)
            }
        });
        return if x > ret.root.key() && *ret.root != TreeNode::Null {
            ret.select(ret.root.child(LCHD).size() + 1)
        } else {
            ret
        }
    }

    fn insert(&self, x: i32) -> Self {
        let origin = self.lowerbound(x);
        let ret = Splay{
            temp: origin.temp,
            root: if origin.root.key() >= x {
                Rc::new(TreeNode::new(x, LCHD, 
                    origin.root.child(LCHD), origin.root.replace_chd(LCHD,
                    TreeNode::Null)))
            } else {
                Rc::new(TreeNode::new(x, LCHD,
                    (*origin.root).clone(),
                    TreeNode::Null))
            }
        };
        ret
    }

    fn remove(&self, x: i32) -> Self {
        let origin = self.lowerbound(x);
        if let TreeNode::Null = origin.root.child(RCHD) {
            return Splay {
                temp: origin.temp,
                root: Rc::new(origin.root.child(LCHD))
            }
        } else {
            let sub = Splay {
                temp: origin.temp,
                root: Rc::new(origin.root.child(RCHD))
            }.select(0);
            return Splay {
                temp: sub.temp,
                root: Rc::new(sub.root.replace_chd(LCHD,
                    origin.root.child(LCHD)))
            }
        }
    }

    fn at(&mut self, k: i32) -> i32 {
        *self = self.select(k);
        self.root.key()
    }

    fn rank(&mut self, x: i32) -> i32 {
        *self = self.lowerbound(x);
        self.root.child(0).size() + 1
    }

    fn new() -> Self {
        Splay {
            root: Rc::new(TreeNode::Null),
            temp: Rc::new(TreeNode::new(0, 0, TreeNode::Null, TreeNode::Null))
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();
    let mut splay = Splay::new();

    stdin.read_line(&mut buf).expect("Invalid input");
    let n: i32 = buf.trim().parse().unwrap();
    for _ in 0..n {
        buf = String::new();
        stdin.read_line(&mut buf).expect("Invalid input");
        let buf_vec: Vec<&str> = buf.trim().split(" ").collect();
        let opt: i32 = buf_vec[0].parse().unwrap();
        let val: i32 = buf_vec[1].parse().unwrap();
        match opt {
            1 => splay = splay.insert(val),
            2 => splay = splay.remove(val),
            3 => println!("{}", splay.rank(val)),
            4 => println!("{}", splay.at(val - 1)),
            5 => {
                splay = splay.lowerbound(val);
                if splay.root.key() >= val {
                    let mut ptr = splay.root.child(0);
                    while let TreeNode::Node{..} = ptr.child(1) {
                        ptr = ptr.child(1);
                    }
                    println!("{}", ptr.key());
                } else {
                    println!("{}", splay.root.key())
                }
            },
            6 => {
                splay = splay.lowerbound(val + 1);
                if splay.root.key() >= val + 1 {
                    println!("{}", splay.root.key())
                }
                else {
                    println!("NONE\n");
                }
            },
            _ => {}
        };
    }
}
