using System.Runtime.InteropServices;
static class Imports {
const string DLL = ".dll";
struct Foo {

};

[DllImport(DLL)]
static extern void extra_debug_fn();

[DllImport(DLL)]
static extern void root(Foo a);
}
