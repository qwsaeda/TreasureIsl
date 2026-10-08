/* ea:   0x0003CB28 */
/* name: sub_3CB28 */
/* size: 112 */
int __fastcall sub_3CB28(int a1, int a2, unsigned __int16 a3, int a4)
{
  int v5; // r0
  int result; // r0
  int v7; // [sp+4h] [bp-Ch] BYREF

  if ( a3 == 1 )
  {
    v5 = a4 + 288;
  }
  else
  {
    v7 = a3;
    v5 = *(_DWORD *)sub_1DD10((_DWORD *)(a4 + 496), &v7);
  }
  *(_DWORD *)(v5 + 176) = 2;
  *(_WORD *)(v5 + 180) = 3;
  result = a1 + 36;
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    return sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  return result;
}

