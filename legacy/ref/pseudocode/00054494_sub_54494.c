/* ea:   0x00054494 */
/* name: sub_54494 */
/* size: 184 */
int __fastcall sub_54494(int a1, unsigned int a2)
{
  void *v4; // r4
  int v5; // r2
  int result; // r0
  int v7; // r5
  char *v8; // r4

  v4 = *(void **)a1;
  v5 = (unsigned __int64)(954437177LL * (*(_DWORD *)(a1 + 8) - *(_DWORD *)a1)) >> 32;
  result = v5 >> 3;
  if ( (v5 >> 3) + ((unsigned int)v5 >> 31) < a2 )
  {
    v7 = (*(_DWORD *)(a1 + 4) - (int)v4) / 36;
    if ( v4 != nullptr )
    {
      v8 = (char *)sub_54638(a1, a2, v4);
      sub_54420((void **)a1);
    }
    else
    {
      v8 = (char *)sub_5445C(a1 + 8, a2);
    }
    *(_DWORD *)a1 = v8;
    *(_DWORD *)(a1 + 4) = &v8[36 * v7];
    *(_DWORD *)(a1 + 8) = &v8[36 * a2];
    return 9 * a2;
  }
  return result;
}

