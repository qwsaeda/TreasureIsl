/* ea:   0x0003AA5C */
/* name: sub_3AA5C */
/* size: 64 */
int __fastcall sub_3AA5C(int a1, int a2)
{
  unsigned int v2; // r0
  unsigned int v4; // [sp+0h] [bp-8h] BYREF

  v2 = a1 << 16;
  if ( HIWORD(v2) == 1 )
    return a2 + 288;
  v4 = HIWORD(v2);
  return *(_DWORD *)sub_1DD10((_DWORD *)(a2 + 496), (int *)&v4);
}

