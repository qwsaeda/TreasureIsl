/* ea:   0x000534A0 */
/* name: sub_534A0 */
/* size: 108 */
_BYTE *__fastcall sub_534A0(int a1, size_t a2, int a3, int a4)
{
  _BYTE *v7; // r7
  int v8; // r3
  _BYTE *v9; // r2

  if ( a2 != 0 )
  {
    v7 = malloc(a2);
    if ( v7 == nullptr )
      v7 = sub_115D8(a2);
  }
  else
  {
    v7 = nullptr;
  }
  v8 = a4 - a3;
  v9 = v7;
  if ( a4 - a3 > 0 )
  {
    do
    {
      --v8;
      *v9 = v9[a3 - (_DWORD)v7];
      ++v9;
    }
    while ( v8 != 0 );
  }
  return v7;
}

