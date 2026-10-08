/* ea:   0x0004A89C */
/* name: sub_4A89C */
/* size: 56 */
int __fastcall sub_4A89C(int result, unsigned __int8 a2)
{
  int v2; // r4

  v2 = result;
  if ( a2 < 0xAu )
  {
    --*(_BYTE *)(a2 + *(_DWORD *)(result + 188));
    result = *(_DWORD *)(result + 200);
    *(_DWORD *)(v2 + 200) = result - 1;
  }
  return result;
}

