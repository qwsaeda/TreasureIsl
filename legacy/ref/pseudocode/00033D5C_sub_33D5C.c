/* ea:   0x00033D5C */
/* name: sub_33D5C */
/* size: 148 */
int __fastcall sub_33D5C(_DWORD *a1, _DWORD *a2, _DWORD *a3)
{
  _DWORD *v4; // r6
  int result; // r0
  _DWORD *v7; // r4

  v4 = a2;
  if ( a2 == *(_DWORD **)(*a1 + 8) && a3 == (_DWORD *)*a1 )
    return sub_30F10(a1);
  while ( 1 )
  {
    result = v4 != a3;
    if ( v4 == a3 )
      break;
    v7 = v4;
    v4 = sub_16A30(v4);
    sub_34AA8(a1, v7);
  }
  return result;
}

