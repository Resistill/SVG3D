p='/home/user/SVG3D/spec/06-facts.md'
s=open(p,encoding='utf-8').read()
old="对精确凸块（`approx=false`），$\\Omega_n$ 是凸块顶点的盒：多面体实体（box、wedge、plate、hull、梁、台阶、凸截面的 sweep、pipe）的 $\\Omega_n$ 等于该对象的紧包围盒（2.1.8）；含圆的实体（cyl、cone、杆、sphere 之外）用 12 边内接棱柱 / 棱台（2.2.3），顶点落在圆上，对象轴对齐时极值相同、$\\Omega_n$ 等于紧包围盒，非轴对齐时 $\\Omega_n$ 比按真实形状算的紧包围盒**小**至多 $R(1 - \\cos 15°)$（2.1.8 的 `tight` 按真实几何，本章的 $\\Omega_n$ 按凸块；两者不要求相等，09-compat.md 的 9.4 对 `tight` 比参考时的额外容许量就是这个差）；球的 $\\Omega_n$ 是 $c \\pm r$，精确。对近似凸块，$\\Omega_n$ 是代理的盒。$\\Omega_n$ 只用于粗筛与扫描序，不输出。"
new="对精确凸块（`approx=false`），$\\Omega_n$ 是凸块顶点（球则 $c \\pm r$）的盒：多面体实体（box、wedge、plate、hull、梁、台阶）与 sphere 的 $\\Omega_n$ 等于该对象的紧包围盒（2.1.8）；圆类实体的凸块用内接多边形代替圆（cyl、cone、杆：12 边棱柱 / 棱台，2.2.3、2.2.8；圆截面的 sweep / pipe：每环取 8 点，2.2.9），顶点落在圆上，cyl / cone / 杆轴对齐时极值与紧包围盒相同，否则 $\\Omega_n$ 比按真实形状算的紧包围盒**小**（cyl / cone / 杆每面至多 $R(1 - \\cos 15°)$）。2.1.8 的 `tight` 按真实几何、本章的 $\\Omega_n$ 按凸块，两者不要求相等；$\\Omega_n$ 只用于粗筛与扫描序，不输出（参考输出的 `tight` 恰是原型按凸块顶点算的盒，对它比较时的额外容许量由 09-compat.md 的 9.4 规定）。对近似凸块，$\\Omega_n$ 是代理的盒。"
assert s.count(old)==1
s=s.replace(old,new)
open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
