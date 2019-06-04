using System.Runtime.InteropServices;
static class Imports {
const string DLL = ".dll";
[DllImport(DLL)]
static extern /// The root of all evil.
void root();
}
