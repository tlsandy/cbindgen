using System.Runtime.InteropServices;
static class Imports {
const string DLL = ".dll";
[DllImport(DLL)]
static extern void first();

[DllImport(DLL)]
static extern void second();
}
