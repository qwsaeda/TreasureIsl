/* ea:   0x0003B2A8 */
/* name: sub_3B2A8 */
/* size: 192 */
int __fastcall sub_3B2A8(int a1, int a2, void *a3, _WORD *a4, _WORD *a5)
{
  _WORD *v8; // r5
  int v9; // r3
  int v10; // r0
  int v11; // r1
  _WORD *v13; // r2
  int v14; // r1

  v8 = a4;
  v9 = a5 - a4;
  v10 = ((int)a3 - a2) >> 1;
  if ( v10 < v9 )
  {
    v13 = &v8[v10];
    if ( v8 != v13 )
    {
      v14 = a2 - (_DWORD)v8;
      do
      {
        *(_WORD *)((char *)v8 + v14) = *v8;
        ++v8;
      }
      while ( v8 != v13 );
    }
    sub_3B41C(a1, a3);
    return a1;
  }
  else
  {
    if ( v8 != a5 )
    {
      v11 = a2 - (_DWORD)v8;
      do
      {
        *(_WORD *)((char *)v8 + v11) = *v8;
        ++v8;
      }
      while ( v8 != a5 );
    }
    sub_13248(a1, (void *)(a2 + 2 * v9), a3);
    return a1;
  }
}

