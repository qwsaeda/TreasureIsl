/* ea:   0x0004E71C */
/* name: sub_4E71C */
/* size: 128 */
int __fastcall sub_4E71C(int a1, int a2, int a3, __int16 a4, _WORD *a5)
{
  _WORD *v9; // r9
  int v10; // r2
  int result; // r0
  unsigned int v12; // r5
  int v13; // r4
  int v14; // r0
  int v15; // t1

  v9 = a5;
  v10 = (unsigned __int16)*a5;
  result = v10 << 16;
  if ( *a5 != 0 )
  {
    do
    {
      v12 = *(_DWORD *)(a1 + 1116);
      v13 = *(_DWORD *)(a1 + 4);
      v14 = sub_1B06C(v12, v10);
      sub_1B150(v12, v13, a2, a3, a4, v14);
      v15 = (unsigned __int16)v9[1];
      ++v9;
      v10 = v15;
      a2 += 16;
      result = v15 << 16;
    }
    while ( (_WORD)v15 != 0 );
  }
  return result;
}

