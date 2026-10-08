/* ea:   0x0003BB08 */
/* name: sub_3BB08 */
/* size: 92 */
_DWORD *__fastcall sub_3BB08(int a1, int a2, int a3, int a4)
{
  unsigned __int16 *v5; // r4
  __int16 v6; // r0
  _DWORD *result; // r0

  v5 = (unsigned __int16 *)(sub_3AA5C(a3, a4) + 136);
  v6 = sub_16C9C(v5, 0);
  sub_32410(v5, v6);
  result = dword_81184;
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    return (_DWORD *)sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  return result;
}

