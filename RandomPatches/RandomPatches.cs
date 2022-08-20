using UnityEngine;
using MelonLoader;
using HarmonyLib;
using ABI.CCK.Components;
using ABI_RC.Core;
using ABI_RC.Core.IO;
using ABI_RC.Core.InteractionSystem;
using System;
using System.Collections;
using System.Collections.Generic;
using System.Reflection;
using ABI_RC.Core.Newton.NewtonEditor.Base;
using System.Reflection.Emit;

[assembly: MelonInfo(typeof(AsyncUnload.Starter), nameof(AsyncUnload), "1.1", "Behemoth")]
[assembly: MelonGame("Alpha Blend Interactive", "ChilloutVR")]

namespace AsyncUnload;
public class Starter : MelonMod
{
    static MelonLogger.Instance logger;
    public override void OnApplicationStart()
    {
        logger = LoggerInstance;

        HarmonyInstance.Patch(
            typeof(CVRObjectLoader).GetNestedType("<InstantiateAvatar>d__25", BindingFlags.NonPublic).GetMethod("MoveNext", BindingFlags.NonPublic | BindingFlags.Instance),
            transpiler: new HarmonyMethod(typeof(Starter), nameof(DestroyPrefab))
        );

        HarmonyInstance.Patch(
            typeof(CVRAvatar).GetMethod(nameof(CVRAvatar.Start), BindingFlags.NonPublic | BindingFlags.Instance),
            prefix: new HarmonyMethod(typeof(Starter), nameof(StartPatch))
        );

        HarmonyInstance.Patch(
            typeof(CVRAnimatorManager).GetMethod(nameof(CVRAnimatorManager.ApplyAdvancedAvatarSettings), BindingFlags.Public | BindingFlags.Instance, null, new Type[] {
                typeof(float[]), typeof(int[]), typeof(bool[]), typeof(bool)
            }, null),
            prefix: new HarmonyMethod(typeof(Starter), nameof(ApplyAdvancedAvatarSettings))
        );

        HarmonyInstance.Patch(
            typeof(CVRTexturePropertyParserManager).GetMethod(nameof(CVRTexturePropertyParserManager.Update), BindingFlags.Public | BindingFlags.Instance),
            prefix: new HarmonyMethod(typeof(Starter), nameof(AudiolinkCripple))
        );

        HarmonyInstance.Patch(
            typeof(NewtonEditorManager).GetMethod(nameof(NewtonEditorManager.Update), BindingFlags.Public | BindingFlags.Instance),
            prefix: new HarmonyMethod(typeof(Starter), nameof(NewtonEditorManagerCripple))
        );
    }

    // Note: This is unused right now
    static bool NewtonEditorManagerCripple(NewtonEditorManager __instance)
    {
        __instance.transform.parent.gameObject.SetActive(false);
        return false;
    }

    // Note: TexturePropertyParser takes an unreasonable amount of time in maps that have it enabled.
    //       Needs investigating!
    static bool AudiolinkCripple(CVRTexturePropertyParserManager __instance)
    {
        __instance.gameObject.SetActive(false);
        return false;
    }

    // Note: This function causes a lot of allocations
    static bool ApplyAdvancedAvatarSettings()
    {
        return false;
    }

    // Note: This function invokes the Garbage Collector for a full sweep.
    //       Patching this out should be fine. The garbage collector will run eventually.
    static bool StartPatch(
        CVRAvatar __instance,
        out IEnumerator __result)
    {
        __result = Start();
        return false;
    }

    static IEnumerator Start()
    {
        logger.Msg("Not doing anything lol");
        yield break;
    }

    static IEnumerable<CodeInstruction> DestroyPrefab(IEnumerable<CodeInstruction> instructions)
    {
        var GameObjectDestroy = typeof(UnityEngine.Object).GetMethod(nameof(UnityEngine.Object.Destroy), BindingFlags.Public | BindingFlags.Static, null, new Type[] { typeof(GameObject) }, null);

        var DestoryLocalPatch = new CodeInstruction[] {
            new CodeInstruction(OpCodes.Ldloc, 14),
            new CodeInstruction(OpCodes.Call, GameObjectDestroy),
        };

        var DestroyRemotePatch = new CodeInstruction[] {
            new CodeInstruction(OpCodes.Ldloc, 19),
            new CodeInstruction(OpCodes.Call, GameObjectDestroy),
        };

        var codes = new List<CodeInstruction>(instructions);
        codes.InsertRange(496, DestoryLocalPatch);
        codes.InsertRange(734, DestroyRemotePatch);
        return codes;
    }
}
