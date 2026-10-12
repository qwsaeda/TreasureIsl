/* ea:   0x0005E970 */
/* name: sub_5E970 */
/* size: 88 */
int __fastcall sub_5E970(int a1, unsigned int a2)
{
  if ( a1 == 0 )
    __debugbreak();
  if ( a2 >= *(_DWORD *)(a1 + 8) )
    __debugbreak();
  if ( (*(_BYTE *)a1 & 1) == 1 )
    return *(_DWORD *)(*(_DWORD *)(a1 + 36) + 4 * a2);
  else
    return *(_DWORD *)(a1 + 28) + *(_DWORD *)(a1 + 24) * a2;
}

