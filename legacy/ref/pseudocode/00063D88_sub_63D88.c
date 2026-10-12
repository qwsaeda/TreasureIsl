/* ea:   0x00063D88 */
/* name: sub_63D88 */
/* size: 72 */
BOOL __fastcall sub_63D88(int a1)
{
  const char *v1; // r0

  v1 = (const char *)(*(int (__fastcall **)(int))(*(_DWORD *)(a1 + 20) + 32))(a1 + 20);
  return v1 != nullptr && strncmp(v1, aLucencypackage, 0x10u) == 0;
}

