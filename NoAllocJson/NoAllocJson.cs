using MelonLoader;
using HarmonyLib;
using ABI_RC.Core.InteractionSystem;
using System;
using System.IO;
using System.Runtime;
using System.Runtime.InteropServices;
using System.Reflection;

[assembly: MelonInfo(typeof(NoAllocJson.Starter), nameof(NoAllocJson), "1.1", "Behemoth")]
[assembly: MelonGame("Alpha Blend Interactive", "ChilloutVR")]

namespace NoAllocJson;
public class Starter : MelonMod
{
    public override void OnApplicationStart()
    {
        var dllName = "fastser.dll";
        var dstPath = "ChilloutVR_Data/Plugins/" + dllName;

        try
        {
            using var resourceStream = Assembly.GetExecutingAssembly().GetManifestResourceStream(dllName);
            using var fileStream = File.Open(dstPath, FileMode.Create, FileAccess.Write);
            resourceStream.CopyTo(fileStream);
        }
        catch (IOException ex)
        {
            LoggerInstance.Error("Failed to copy native library: " + ex.Message);
            return;
        }

        IntPtr NativeLibrary = LoadLibraryA(dstPath);
        if (NativeLibrary == IntPtr.Zero)
        {
            var error = Marshal.GetLastWin32Error();
            LoggerInstance.Error("Native library load failed, mod won't work: {0}", error);
            return;
        }

        HarmonyInstance.Patch(
            typeof(CVR_MenuManager).GetMethod("SendCoreUpdate", BindingFlags.NonPublic | BindingFlags.Instance),
            prefix: new HarmonyMethod(typeof(Patch), "SendCoreUpdatePatch")
        );
    }

    [DllImport("kernel32", CharSet = CharSet.Ansi, ExactSpelling = true, SetLastError = true)]
    static extern IntPtr LoadLibraryA(string libName);
}
