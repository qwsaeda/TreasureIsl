/* ea:   0x000305A8 */
/* name: sub_305A8 */
/* size: 72 */
_DWORD *__fastcall sub_305A8(int a1, _DWORD *a2, int a3)
{
  _DWORD *v4; // r4
  int v5; // r3
  int v6; // r1

  v4 = a2;
  v5 = (*(_DWORD *)(a1 + 4) - a3) >> 2;
  if ( v5 > 0 )
  {
    v6 = a3 - (_DWORD)a2;
    do
    {
      --v5;
      *v4 = *(_DWORD *)((char *)v4 + v6);
      ++v4;
    }
    while ( v5 != 0 );
  }
  *(_DWORD *)(a1 + 4) = v4;
  return a2;
}

