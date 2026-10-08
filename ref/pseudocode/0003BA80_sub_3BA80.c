/* ea:   0x0003BA80 */
/* name: sub_3BA80 */
/* size: 68 */
_DWORD *__fastcall sub_3BA80(int a1, int a2, int a3, int a4)
{
  int v5; // r0
  _DWORD *result; // r0

  v5 = sub_3AA5C(a3, a4);
  sub_52480(v5, 2000);
  result = dword_81184;
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    return (_DWORD *)sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  return result;
}

