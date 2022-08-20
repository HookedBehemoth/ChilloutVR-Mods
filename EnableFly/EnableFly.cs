using MelonLoader;

[assembly: MelonInfo(typeof(EnableFly.Starter), nameof(EnableFly), "1.0", "")]
[assembly: MelonGame("Alpha Blend Interactive", "ChilloutVR")]

namespace EnableFly;
public class Starter : MelonMod
{
    public override void OnUpdate()
    {
        var instance = ABI_RC.Systems.MovementSystem.MovementSystem.Instance;
        if (instance != null)
        {
            instance.canFly = true;
            if (UnityEngine.Mathf.Approximately(instance.floatSpeedMultiplier, 0.0f))
            {
                instance.floatSpeedMultiplier = 2.0f;
            }
        }
    }
}
