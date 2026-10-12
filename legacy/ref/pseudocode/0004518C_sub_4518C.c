/* ea:   0x0004518C */
/* name: sub_4518C */
/* size: 168 */
char *__fastcall sub_4518C(int a1, unsigned int a2)
{
  _DWORD *v4; // r5
  char *result; // r0
  _DWORD *v6; // r4
  int v7; // r7
  int v8; // r0
  _BYTE *v9; // r0
  _BYTE *v10; // r6
  int v11; // r2
  bool v12; // zf
  _DWORD *v13; // r1
  char *v14; // r6

  v4 = *(_DWORD **)a1;
  result = *(char **)(a1 + 8);
  if ( a2 > (result - (char *)v4) >> 2 )
  {
    v6 = *(_DWORD **)(a1 + 4);
    v7 = v6 - v4;
    v8 = a1 + 8;
    if ( v4 != nullptr )
    {
      v9 = sub_132C8(v8, a2);
      v10 = v9;
      if ( v4 != v6 )
      {
        v11 = v9 - (_BYTE *)v4;
        do
        {
          v13 = (_DWORD *)((char *)v4 + v11);
          v12 = (_DWORD *)((char *)v4 + v11) == nullptr;
          if ( (_DWORD *)((char *)v4 + v11) != nullptr )
            v9 = (_BYTE *)*v4;
          ++v4;
          if ( !v12 )
            *v13 = v9;
        }
        while ( v4 != v6 );
      }
      sub_132B4((void **)a1);
      *(_DWORD *)a1 = v10;
      result = &v10[4 * v7];
      *(_DWORD *)(a1 + 4) = result;
      *(_DWORD *)(a1 + 8) = &v10[4 * a2];
    }
    else
    {
      v14 = (char *)sub_132C8(v8, a2);
      result = &v14[4 * v7];
      *(_DWORD *)a1 = v14;
      *(_DWORD *)(a1 + 4) = result;
      *(_DWORD *)(a1 + 8) = &v14[4 * a2];
    }
  }
  return result;
}

