/* ea:   0x00039CC0 */
/* name: sub_39CC0 */
/* size: 180 */
int __fastcall sub_39CC0(_DWORD **a1, _DWORD **a2, unsigned __int8 (__fastcall *a3)(_DWORD, _DWORD))
{
  _DWORD **v4; // r10
  _DWORD **v5; // r9
  unsigned __int8 v6; // r8
  _DWORD *v7; // r5
  _DWORD *v8; // r7
  _DWORD *v9; // r4
  int result; // r0

  v4 = (_DWORD **)*a1;
  v5 = (_DWORD **)*a2;
  v6 = 1;
  v7 = (_DWORD *)**a1;
  v8 = (_DWORD *)**a2;
  while ( v7 != v4 && v8 != v5 )
  {
    if ( a3(v8[2], v7[2]) != 0 )
    {
      v9 = (_DWORD *)*v8;
      sub_39D90(v7, v8);
      v8 = v9;
    }
    else
    {
      v7 = (_DWORD *)*v7;
    }
  }
  if ( v8 == v5 )
    v6 = 0;
  result = v6;
  if ( v6 != 0 )
    return sub_39D90(v4, v8);
  return result;
}

