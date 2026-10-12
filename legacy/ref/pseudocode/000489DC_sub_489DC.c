/* ea:   0x000489DC */
/* name: sub_489DC */
/* size: 116 */
unsigned int __fastcall sub_489DC(unsigned int result, int a2, char a3)
{
  unsigned int v3; // r4
  int v5; // r0
  unsigned int *v6; // r4
  int v7; // t1
  unsigned int v8; // r3
  unsigned int v9; // r2
  unsigned int *v10; // r5
  unsigned int v11; // t1

  v3 = result;
  if ( a2 != 0 && a3 != 0 )
  {
    v5 = sub_16C9C((unsigned __int16 *)(a2 + 136), 2);
    v7 = *(_DWORD *)(v3 + 144);
    v6 = (unsigned int *)(v3 + 144);
    v8 = (v5 + 4) / 5u;
    v9 = v6[1];
    *v6 = v7 + v8;
    if ( v7 + v8 > v9 )
      *v6 = v9;
    v11 = *(_DWORD *)(a2 + 144);
    v10 = (unsigned int *)(a2 + 144);
    result = v11;
    if ( v8 < v11 )
    {
      result -= v8;
      *v10 = result;
    }
    else
    {
      *v10 = 0;
    }
  }
  return result;
}

