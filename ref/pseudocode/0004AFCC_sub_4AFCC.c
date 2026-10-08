/* ea:   0x0004AFCC */
/* name: sub_4AFCC */
/* size: 180 */
int __fastcall sub_4AFCC(_DWORD ***a1, int ***a2, unsigned __int8 (__fastcall *a3)(int, _DWORD))
{
  _DWORD **v4; // r10
  int **v5; // r9
  unsigned __int8 v6; // r8
  _DWORD *v7; // r5
  int *v8; // r7
  int v9; // r4
  int result; // r0

  v4 = *a1;
  v5 = *a2;
  v6 = 1;
  v7 = **a1;
  v8 = **a2;
  while ( v7 != v4 && v8 != (int *)v5 )
  {
    if ( a3(v8[2], v7[2]) != 0 )
    {
      v9 = *v8;
      sub_39D90((int)v7, (int)v8, *v8);
      v8 = (int *)v9;
    }
    else
    {
      v7 = (_DWORD *)*v7;
    }
  }
  if ( v8 == (int *)v5 )
    v6 = 0;
  result = v6;
  if ( v6 != 0 )
    return sub_39D90((int)v4, (int)v8, (int)v5);
  return result;
}

