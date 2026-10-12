/* ea:   0x000173C4 */
/* name: sub_173C4 */
/* size: 76 */
int __fastcall sub_173C4(int a1, int a2)
{
  int *v3; // r0
  bool v4; // zf
  int result; // r0
  char v6; // [sp+0h] [bp-10h] BYREF

  v3 = (int *)sub_17410(*(_DWORD *)(a1 + 8), *(_DWORD *)(a1 + 12), a2, &v6);
  v4 = v3 == *(int **)(a1 + 12);
  result = *v3;
  if ( v4 )
    __debugbreak();
  return result;
}

