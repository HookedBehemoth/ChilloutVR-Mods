using System;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using ABI_RC.Core;
using HarmonyLib;
using MelonLoader;

[assembly: MelonInfo(typeof(FastDecrypt.Starter), nameof(FastDecrypt), "1.1", "Behemoth")]
[assembly: MelonGame("Alpha Blend Interactive", "ChilloutVR")]

namespace FastDecrypt;

public class Starter : MelonMod
{
    static IntPtr NativeLibrary;
    static DecryptDelegate Decrypt;

#if DEBUG
    public static bool enabled = true;
    static MelonLogger.Instance Logger;
#endif

    public override void OnApplicationStart()
    {
#if DEBUG
        Logger = LoggerInstance;

        if (!enabled)
        {
            Logger.Msg("FastDecrypt is disabled");
            HarmonyInstance.Patch(
                typeof(CVRTools).GetMethod(nameof(CVRTools.decrypt)),
                prefix: new HarmonyMethod(typeof(TimePatch).GetMethod(nameof(TimePatch.Prefix))),
                postfix: new HarmonyMethod(typeof(TimePatch).GetMethod(nameof(TimePatch.Postfix)))
            );
            return;
        }
#endif

        var dllName = "libdec.dll";
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

        NativeLibrary = LoadLibraryA(dstPath);
        if (NativeLibrary == IntPtr.Zero)
        {
            var error = Marshal.GetLastWin32Error();
            LoggerInstance.Error("Native library load failed, mod won't work: {0}", error);
            return;
        }

        Decrypt = Marshal.GetDelegateForFunctionPointer<DecryptDelegate>(GetProcAddress(NativeLibrary, "decrypt"));
        if (Decrypt == null)
        {
            LoggerInstance.Error("Native library load failed, mod won't work: failed to find decrypt function");
            return;
        }

        HarmonyInstance.Patch(
            typeof(CVRTools).GetMethod(nameof(CVRTools.decrypt)),
            new HarmonyMethod(typeof(DecryptPatch), nameof(DecryptPatch.Prefix))
        );
    }

    public class DecryptPatch
    {
        public static bool Prefix(out byte[] __result, string guid, byte[] bytes, byte[] keyFrag)
        {

#if DEBUG
            var timer = new Stopwatch();
            timer.Start();
#endif

            __result = new byte[bytes.Length + keyFrag.Length];
            unsafe
            {
                fixed (byte* b = bytes, k = keyFrag, d = __result)
                {
                    Decrypt(guid, (nuint)guid.Length, b, (nuint)bytes.Length, k, (nuint)keyFrag.Length, d);
                }
            }

#if DEBUG
            timer.Stop();
            Logger.Msg("{0}: Decryption took {1}ms", guid, timer.Elapsed.TotalMilliseconds);
#endif

            return false;
        }
    }

    [UnmanagedFunctionPointer(CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public unsafe delegate void DecryptDelegate(string guid_ptr, nuint guid_len, byte* data_ptr, nuint data_len, byte* key_ptr, nuint key_len, byte* result_ptr);

    [DllImport("kernel32", CharSet = CharSet.Ansi, ExactSpelling = true, SetLastError = true)]
    static extern IntPtr GetProcAddress(IntPtr hModule, string procName);

    [DllImport("kernel32", CharSet = CharSet.Ansi, ExactSpelling = true, SetLastError = true)]
    static extern IntPtr LoadLibraryA(string libName);

#if DEBUG
    public class TimePatch
    {
        public static bool Prefix(out Stopwatch __state)
        {
            __state = new Stopwatch();
            __state.Start();
            return true;
        }
        public static void Postfix(ref Stopwatch __state, string guid)
        {
            __state.Stop();
            Logger.Msg("{0}: Decryption took {1}ms", guid, __state.ElapsedMilliseconds);
        }
    }
#endif
}
