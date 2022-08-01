using MelonLoader;
using UnityEngine;

[assembly: MelonInfo(typeof(NoVideo.Starter), nameof(NoVideo), "1.0", "Behemoth")]
[assembly: MelonGame("Alpha Blend Interactive", "ChilloutVR")]

namespace NoVideo;
public class Starter : MelonMod
{
    public override void OnSceneWasLoaded(int buildIndex, string sceneName)
    {
        if (sceneName == "Login")
        {
            GameObject.Find("LoginCanvas/BG (1)").SetActive(false);
            GameObject.Find("LoginCanvas/BG").SetActive(false);
        }
    }
}
