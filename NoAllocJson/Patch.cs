using System.Runtime.CompilerServices;
using ABI_RC.Core.InteractionSystem;
using MelonLoader;
using cohtml;

namespace NoAllocJson;
public class Patch
{

    static string buffer = new string('\0', 0x4000);
    static int count = 0;
    static bool enabled = true;
    static bool SendCoreUpdatePatch(CVR_MenuManager __instance)
    {
        if (__instance._quickMenuReady)
        {
            if (count == 0) {
                var watch = new System.Diagnostics.Stopwatch();
                watch.Start();
                var expected = UnityEngine.JsonUtility.ToJson(__instance.coreData);
                watch.Stop();
                var unity = watch.Elapsed;
                MelonLogger.Msg("Want (" + unity.TotalMilliseconds + "): " + expected);
                watch.Reset();
                watch.Start();
                var serialized = SerializeInplace(__instance.coreData, buffer);
                watch.Stop();
                var us = watch.Elapsed;
                MelonLogger.Msg("Got (" + us.TotalMilliseconds + "): " + buffer);
                __instance.quickMenu.View.TriggerEvent("ReceiveCoreUpdate", buffer);
                count += 1;
            } else {
                // SerializeInplace(__instance.coreData, buffer);
                if (enabled) {
                    SerializeInplace(__instance.coreData, buffer);
                    __instance.quickMenu.View.TriggerEvent("ReceiveCoreUpdate", buffer);
                } else {
                    var expected = UnityEngine.JsonUtility.ToJson(__instance.coreData);
                    __instance.quickMenu.View.TriggerEvent("ReceiveCoreUpdate", expected);
                }
            }
        }
        if (__instance.gameRulesUpdated && __instance._quickMenuReady)
        {
            __instance.quickMenu.View.TriggerEvent("ReceiveGameRuleUpdate");
            __instance.gameRulesUpdated = false;
        }
        if (__instance.avatarUpdated && __instance._quickMenuReady)
        {
            __instance.quickMenu.View.TriggerEvent("ReceiveAvatarUpdate");
            __instance.avatarUpdated = false;
        }
        return false;
    }
    [MethodImplAttribute(MethodImplOptions.InternalCall)]
    public extern static nuint SerializeInplace(CVR_Menu_Data data, string buffer);
}
