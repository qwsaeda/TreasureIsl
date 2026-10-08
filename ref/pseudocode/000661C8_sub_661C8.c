/* ea:   0x000661C8 */
/* name: sub_661C8 */
/* size: 144 */
char *__fastcall sub_661C8(int a1, unsigned int a2)
{
  _BYTE *v4; // r2
  char *result; // r0
  int v6; // r5
  char *v7; // r4

  v4 = *(_BYTE **)a1;
  result = *(char **)(a1 + 8);
  if ( a2 > (result - v4) >> 2 )
  {
    v6 = (*(_DWORD *)(a1 + 4) - (int)v4) >> 2;
    if ( v4 != nullptr )
    {
      v7 = (char *)sub_66480(a1, a2);
      if ( *(_DWORD *)a1 != 0 )
      {
        free(*(void **)a1);
        *(_DWORD *)a1 = v7;
        result = &v7[4 * v6];
        *(_DWORD *)(a1 + 4) = result;
        *(_DWORD *)(a1 + 8) = &v7[4 * a2];
        return result;
      }
    }
    else
    {
      v7 = (char *)sub_65EA4(a1 + 8, a2);
    }
    result = &v7[4 * v6];
    *(_DWORD *)a1 = v7;
    *(_DWORD *)(a1 + 4) = result;
    *(_DWORD *)(a1 + 8) = &v7[4 * a2];
  }
  return result;
}

