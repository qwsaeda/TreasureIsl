/* ea:   0x00043C5C */
/* name: sub_43C5C */
/* size: 104 */
int __fastcall sub_43C5C(int a1)
{
  int v2; // r5
  int v3; // r4
  void *v4; // r0
  int result; // r0

  if ( a1 != -8 )
  {
    v2 = 0;
    if ( *(int *)(a1 + 492) > 0 )
    {
      v3 = a1;
      do
      {
        v4 = *(void **)(v3 + 8);
        if ( v4 == nullptr )
          __debugbreak();
        free(v4);
        *(_DWORD *)(v3 + 8) = 0;
        *(_DWORD *)(v3 + 496) = 0;
        ++v2;
        v3 += 4;
      }
      while ( v2 < *(_DWORD *)(a1 + 492) );
    }
  }
  result = sub_44AAC(a1 + 976);
  *(_DWORD *)(a1 + 492) = 0;
  return result;
}

