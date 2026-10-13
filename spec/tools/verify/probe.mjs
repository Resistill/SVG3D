import { compile } from '/home/user/resistill/svg3d-prototype/src/s3d.js';
const cases = {
 b_copyblock_on_repeat: `box ground size=40,1,40 pin=top
repeat row n=3 step=4,0,0 on=ground { box tank size=1,2,1 }
copy row2 of=row on=ground[0,1,0.5] { cyl post r=0.2 h=1 on=tank }`,
 c_copy_of_shadow: `box ground size=40,1,40 pin=top
group g on=ground { box a size=1,1,1; box b size=1,1,1 on=a }
mirror g2 of=g across=(5,0,0) axis=x
copy k of=g2.b at=10,0,0`,
 d_over_copy_node_across: `box a size=1,1,1
mirror a2 of=a across=(3,0,0)
over a2 across=(5,0,0)`,
 e_over_noop: `box a size=1,1,1
over a`,
 f_inplace_lost_on_rebuild: `def t(n=1) { box b size=n,1,1 }
t x at=3,0,0
over x color=#ff0000 turn=45
over x n=2`,
 g_copy_order: `box a size=1,1,1
copy c of=a at=5,0,0
box b size=1,1,1 at=9,0,0
group g { box p size=1,1,1 ; copy q of=p at=2,0,0 ; box r size=1,1,1 at=4,0,0 ; box ; box }`,
 i_derived_redeclare: `def t(n=1) { box b size=n,1,1 }
def u(n) : t { box c size=1,1,1 on=b }
def v(n=5) : t(n=3) { }
u x n=2 at=0,0,0
v y at=10,0,0
u z at=20,0,0`,
 j_scope_filter: `box ground size=40,1,40 pin=top
box far size=1,1,1 at=10,0,0 #t
group g on=ground { box near size=1,1,1 #t ; over #t size=2,2,2 }`,
 k_copy_of_instance: `repeat row n=3 step=4,0,0 { let s=1+i ; box tank size=s,2,1 }
copy one of=row.2 at=0,0,10`,
 l_nested_copy: `box a size=1,1,1
group g { copy a2 of=a at=2,0,0 }
copy g2 of=g at=0,0,10`,
 n_drop_shadow: `group g { box a size=1,1,1; box b size=1,1,1 at=2,0,0 }
mirror g2 of=g across=(5,0,0) axis=x
drop g2.b
over g2.a { box extra size=0.5,0.5,0.5 on=it }`,
 o_over_mirrored_group: `mirror m across=(5,0,0) axis=x { box a size=1,1,1 }
over m.mirrored { box extra size=0.5,0.5,0.5 at=0,0,3 }
over m.mirrored.a size=2,2,2`,
 p_copy_in_over_block: `box a size=1,1,1
group g at=0,0,10 { box p size=1,1,1 }
over g { copy a2 of=a at=3,0,0 }`,
 q_over_tmpl_param_on_shadow: `def t(n=1) { box b size=n,1,1 }
t x
mirror x2 of=x across=(5,0,0)
over x2 n=2
over x2.b : cyl r=1
over x2.b size=3,1,1`,
 r_retype: `box ground size=40,1,40 pin=top
box hall size=10,6,8 on=ground turn=30 mat=none
over hall : subtract { box outer size=10,6,8 }
box s size=2,2,2 at=20,0,0 pin=center gap=1 turn=10 #k ~m=1
over s : cyl r=1
def shed(w=4) { box body size=w,3,w }
def big(h=5) : shed(w=6) { box top size=1,h,1 on=body }
shed h1 w=5 at=0,0,20 mat=none
over h1 : big
big h2 at=0,0,30
over h2 : shed
def other(w=1, q=2) { box z size=w,q,1 }
shed h3 w=7 at=0,0,40
over h3 : other
group gg at=30,0,0 { box inner size=1,1,1 }
over gg : box size=2,2,2
over gg : group { box j size=1,1,1 }`,
 s_rebuild_drops_prior: `repeat r n=3 step=2,0,0 { box t size=1,1,1 }
drop r.1
over r { sphere s r=0.2 on=t }
over r n=4`,
 t_over_copynode_props: `box a size=1,1,1
mirror a2 of=a across=(3,0,0)
over a2 mat=none
over a2 size=2,2,2`,
 u_rotate_n_over: `rotate rr n=2 { box blade size=1,3,0.2 at=2,0,0 }
over rr n=4`,
};
for (const [name, src] of Object.entries(cases)) {
  const r = compile(src);
  console.log(`=== ${name}`);
  for (const n of r.nodes) console.log(`  ${n.path}\t${n.kind}\tghost=${n.ghost}\tof=${n.of}\ttypes=${JSON.stringify(n.types)}\ttags=${JSON.stringify(n.tags)}\tmeta=${JSON.stringify(n.meta)}\tbox=${n.box ? n.box.min.map((v)=>+v.toFixed(2))+'..'+n.box.max.map((v)=>+v.toFixed(2)) : '-'}`);
  for (const it of r.items) if (it.material?.color==='#ff0000'||it.material?.name==='none') console.log(`  item ${it.path} mat=${JSON.stringify(it.material)}`);
  console.log('  errors:', JSON.stringify(r.errors.map((e)=>`${e.line}: ${e.message}`)));
}
