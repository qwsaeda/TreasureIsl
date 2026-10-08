/* ea:   0x0003CBB8 */
/* name: sub_3CBB8 */
/* size: 92 */
__int16 **__fastcall sub_3CBB8(int a1, int a2, int a3, int a4)
{
  int v6; // r0
  __int16 **result; // r0

  v6 = sub_3AA5C(a3, a4);
  sub_51F24(v6);
  result = (__int16 **)(a1 + 36);
  *(_DWORD *)(a4 + 188) += 20;
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    return (__int16 **)sub_17890((int)dword_81184, *result, *(__int16 **)(a1 + 40));
  return result;
}

