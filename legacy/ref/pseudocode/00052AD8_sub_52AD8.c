/* ea:   0x00052AD8 */
/* name: sub_52AD8 */
/* size: 192 */
char *__fastcall sub_52AD8(int a1, size_t a2)
{
  _BYTE *v4; // r2
  char *result; // r0
  int v6; // r5
  char *v7; // r4
  char *v8; // r4

  v4 = *(_BYTE **)a1;
  result = *(char **)(a1 + 8);
  if ( result - v4 < a2 )
  {
    v6 = *(_DWORD *)(a1 + 4) - (_DWORD)v4;
    if ( v4 != nullptr )
    {
      v7 = (char *)sub_534A0(a1, a2);
      if ( *(_DWORD *)a1 != 0 )
      {
        free(*(void **)a1);
        *(_DWORD *)a1 = v7;
        *(_DWORD *)(a1 + 4) = &v7[v6];
        *(_DWORD *)(a1 + 8) = &v7[a2];
        return &v7[v6];
      }
    }
    else if ( a2 != 0 )
    {
      v7 = (char *)malloc(a2);
      if ( v7 == nullptr )
      {
        v8 = (char *)sub_115D8(a2);
        *(_DWORD *)a1 = v8;
        *(_DWORD *)(a1 + 4) = &v8[v6];
        *(_DWORD *)(a1 + 8) = &v8[a2];
        return &v8[v6];
      }
    }
    else
    {
      v7 = nullptr;
    }
    *(_DWORD *)a1 = v7;
    *(_DWORD *)(a1 + 4) = &v7[v6];
    *(_DWORD *)(a1 + 8) = &v7[a2];
    return &v7[v6];
  }
  return result;
}

