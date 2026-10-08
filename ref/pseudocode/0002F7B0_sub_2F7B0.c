/* ea:   0x0002F7B0 */
/* name: sub_2F7B0 */
/* size: 100 */
BOOL __fastcall sub_2F7B0(_DWORD ***a1, _DWORD *a2)
{
  _DWORD **v3; // r5
  _DWORD *i; // r2
  BOOL result; // r0
  _DWORD *v6; // r4
  _DWORD *v7; // r0

  v3 = *a1;
  for ( i = **a1; ; i = v6 )
  {
    result = i != v3;
    if ( i == v3 )
      break;
    v6 = (_DWORD *)*i;
    if ( *a2 == i[2] )
    {
      v7 = (_DWORD *)i[1];
      *v7 = v6;
      v6[1] = v7;
      free(i);
    }
  }
  return result;
}

