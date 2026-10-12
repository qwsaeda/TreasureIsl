/* ea:   0x00031150 */
/* name: sub_31150 */
/* size: 112 */
int __fastcall sub_31150(int a1, int a2)
{
  _DWORD *v2; // r2
  int v3; // r5

  v2 = *(_DWORD **)(a1 + 28);
  v3 = 0;
  while ( v2 != *(_DWORD **)(a1 + 32) )
  {
    if ( *v2 == a2 )
    {
      ++v3;
      if ( a2 != 1 && a2 != 3 && a2 != 0 )
        return v3;
    }
    ++v2;
  }
  if ( a2 == 0 )
    return *(unsigned __int8 *)(a1 + 49) + v3 - 4;
  return v3;
}

