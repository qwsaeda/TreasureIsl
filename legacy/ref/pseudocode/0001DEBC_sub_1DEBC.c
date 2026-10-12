/* ea:   0x0001DEBC */
/* name: sub_1DEBC */
/* size: 144 */
int __fastcall sub_1DEBC(int a1, unsigned int a2)
{
  void *v4; // r2
  int result; // r0
  int v6; // r5
  int v7; // r4

  v4 = *(void **)a1;
  result = *(_DWORD *)(a1 + 8);
  if ( a2 > (result - (int)v4) >> 2 )
  {
    v6 = (*(_DWORD *)(a1 + 4) - (int)v4) >> 2;
    if ( v4 != nullptr )
    {
      v7 = sub_1EA28(a1, a2);
      if ( *(_DWORD *)a1 != 0 )
      {
        free(*(void **)a1);
        *(_DWORD *)a1 = v7;
        result = v7 + 4 * v6;
        *(_DWORD *)(a1 + 4) = result;
        *(_DWORD *)(a1 + 8) = v7 + 4 * a2;
        return result;
      }
    }
    else
    {
      v7 = sub_1E598(a1 + 8, a2);
    }
    result = v7 + 4 * v6;
    *(_DWORD *)a1 = v7;
    *(_DWORD *)(a1 + 4) = result;
    *(_DWORD *)(a1 + 8) = v7 + 4 * a2;
  }
  return result;
}

