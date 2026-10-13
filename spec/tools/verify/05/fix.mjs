import fs from 'node:fs';
const f = '/home/user/SVG3D/spec/05-layers.md';
let t = fs.readFileSync(f, 'utf8');
const rep = (a, b) => { const n = t.split(a).length - 1; if (n !== 1) { console.error('NOT UNIQUE/FOUND (' + n + '): ' + a.slice(0, 80)); process.exit(1); } t = t.replace(a, b); };

// E1 blocker: appearance-timing over with use + template params (4.5.1 第 4 条)
rep('| **外观时机**（外观层） | `style`；`variant`；外观时机的 `over`（最外层、无块、无换形、至少一个属性且键 ⊆ `{mat, color, use}`） | 8 |',
    '| **外观时机**（外观层） | `style`；`variant`；外观时机的 `over`（最外层、无块、无换形、至少一个属性；没有 `use` 时每个键都属于 `{mat, color}`，有 `use` 时其余键是 `mat` / `color` 或传给模板的参数——4.5.1 第 4 条；`over roof use=ribbed w=10` 也是外观时机的） | 8 |');
rep('几何时机的 `over`（4.5.1：带块、带换形 `: T`、含 `{mat, color, use}` 之外的键，或写在任何块里的）；`drop`',
    '几何时机的 `over`（4.5.1：带块、带换形 `: T`、没有 `use` 而含 `{mat, color}` 之外的键，或写在任何块里的）；`drop`');
rep('`over a mat=steel` 即使 a 本来就是 steel 也是外观时机。实现**不得**',
    '`over a mat=steel` 即使 a 本来就是 steel 也是外观时机；`over a use=T w=3` 是外观时机（`w` 是传给 T 的参数），`w` 不是 T 的参数时仍是外观时机、在登记时报错"模板 T 没有参数 w"（4.5.1 第 4 条、7.8.8）。实现**不得**');
rep('over #cargo use=crate_skin                     // 外观层：等价于 variant #cargo use=crate_skin\n',
    'over #cargo use=crate_skin                     // 外观层：等价于 variant #cargo use=crate_skin\nover #cargo use=crate_skin w=2                 // 外观层：use 之外的键是模板参数（4.5.1 第 4 条）\n');
rep('| 4.5.1 / 5.4.1：两条 `over` 都是外观时机（最外层、无块、无换形、键 ⊆ {mat, color, use}） |',
    '| 4.5.1 / 5.4.1：两条 `over` 都是外观时机（最外层、无块、无换形、键分别只有 `use`、只有 `color`） |');

// E2: invalid UTF-8 is a lexical error (1.2.1), not "找不到文件"
rep('4. 读不到文件（不存在、不是普通文件、不可读、不是合法的 UTF-8）→ 报错"import 找不到文件 $I$"，不导入。',
    '4. 读不到文件（不存在、不是普通文件、不可读）→ 报错"import 找不到文件 $I$"，不导入。读到的字节里无法解码的序列不属于本条：按 1.2.1 节作为 G 的语法错误报（归属于 G、该行丢弃），文件照常加载。');

// E3: material error rules align with 10.1.1 / 10.1.2
rep('- 出错的 material 语句（属性不合法、位置不合法，1.3.7 与 1.4.6 节）不登记、不改变有效定义。块内、模板体内的 `material` 报错"material 只能写在最外层"（D6；原型静默忽略）。',
    '- 位置不合法或缺名字的 material 语句整条无效：不登记、不改变有效定义（10.1.1）；块内、模板体内的 `material` 报错"material 只能写在最外层"（D6；原型静默忽略）。属性级的错误（未知键、类型 / 范围错误、常量求值失败，1.4.6 节）只使出错的那个属性视为没写，该语句**仍登记**并照常替换同名的先前定义（10.1.2）。');
rep('| 同名 `material` | 后写整体替换 | 出错的不登记 | 否 |',
    '| 同名 `material` | 后写整体替换 | 位置错误 / 缺名字的不登记；属性级错误的仍登记（10.1.2） | 否 |');

// E4: top-level let evaluates in the global frame
rep('最终定义的表达式在它自己的语句的上下文里求值（它所在文件的帧、根作为搜索起点）',
    '最终定义的表达式在它自己的语句的上下文里求值（全局帧、根作为搜索起点，1.5.8 节）');

// E5: prototype behaviour on file header
rep('| 5.1.4 | 被 import 文件的 `lang` | 不检查 | 不是 2 时报"不支持的语言版本"，该文件不编译 | 8.1.5（D7） |',
    '| 5.1.4 | 被 import 文件的 `lang` | loadProgram 不检查 `lang`；文件头行本身在原型里报"未知的类型 “~lang=2”"（makeStmt，9.1.2），不论在哪个文件 | 不是 2 时报"不支持的语言版本"，该文件不编译 | 8.1.5（D7） |');

// E6: facts fields
rep('3. **事实**：`facts[]` 的条数、顺序与每条的 `rel` `a` `b` `text` `gap` `eps` `provider` 相同；`factsSkipped`、`factProviders` 相同。',
    '3. **事实**：`facts[]` 的条数、顺序与每条的全部字段（6.6.1；参考输出里是 `rel` `a` `b` `text` `gap` `eps` `provider`）相同；`factsSkipped`、`factProviders` 相同。');

// E7: hidden applies to N and its descendants
rep('对表里每个节点 $N$ 打 `hidden` 标记，在 $N$ 的子节点末尾挂一个段为 `skin` 的子树并求解它。$N$ 仍在树里',
    '对表里每个节点 $N$ 及其当时的全部后代打 `hidden` 标记（4.7 第 1 条），在 $N$ 的子节点末尾挂一个段为 `skin` 的子树并求解它。$N$ 仍在树里');

// E8: second difference per 7.7.4
rep('它与 style / variant 的唯一差别在**空选择**：',
    '它与 style / variant 的差别只在**空选择**（以及错误携带的语句种类，7.7.4 第 2 条）：');

// E9: prototype duplicate example wording
rep('得到 13 个节点、路径两两重复、没有任何错误；',
    '得到 13 个节点（除被 `drop h2` 删掉的那一份 h2 外，每个路径出现两次）、没有任何错误；');

// E10: changed-node list wording
rep('`arm`、`arm2`、`tower.decks.*`、`tower2.decks.*`、`hangar.body.opening`、`hangar.body.arch`、`tanks.*`、`feedline1/2`、`feed1/2`',
    '`arm`、`arm2`、`tower.decks`、`tower2.decks`、`tanks` 与它们的全部实例、`hangar.body.opening`、`hangar.body.arch`、`feedline1/2`、`feed1/2`');

// E11: per-file output shape belongs to 08
rep('它按文件记录（08-metadata.md）。',
    '它按文件记录，随 08-metadata.md 的 8.4.3 规定的按文件输出给出（输出形状由该节定义，本章不定义）。');
rep('以及 1.5.6 的警告"kind() 不认识 X"（新增的 def 使 X 成为已知模板名）。',
    '以及 1.5.6 的警告"kind() 不认识 X"（新增的 def 使 X 成为已知模板名）；8.4.3 的按文件输出只在参与编译的文件集合或各文件的文件头改变时不同。');

fs.writeFileSync(f, t);
console.log('ok, CRLF?', t.includes('\r'));
