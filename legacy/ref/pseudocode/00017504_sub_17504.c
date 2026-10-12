/* ea:   0x00017504 */
/* name: sub_17504 */
/* size: 76 */
int __fastcall sub_17504(int a1, int a2)
{
  int *v3; // r0
  bool v4; // zf
  int result; // r0
  char v6; // [sp+0h] [bp-10h] BYREF

  v3 = (int *)sub_17550(*(_DWORD *)(a1 + 4), *(_DWORD *)(a1 + 8), a2, &v6);
  v4 = v3 == *(int **)(a1 + 8);
  result = *v3;
  if ( v4 )
    __debugbreak();
  return result;
}

