/* ea:   0x00039820 */
/* name: sub_39820 */
/* size: 160 */
int __fastcall sub_39820(int *a1, unsigned __int16 *a2)
{
  char v2; // r5
  int v3; // r7
  int v4; // r3

  v2 = 1;
  v3 = *a1;
  v4 = *(_DWORD *)(*a1 + 4);
  while ( v4 != 0 )
  {
    if ( *a2 > (unsigned int)*(unsigned __int16 *)(v4 + 16) )
    {
      v4 = *(_DWORD *)(v4 + 12);
    }
    else
    {
      v3 = v4;
      v4 = *(_DWORD *)(v4 + 8);
    }
  }
  if ( v3 == *a1 )
    return *a1;
  if ( *a2 >= (unsigned int)*(unsigned __int16 *)(v3 + 16) )
    v2 = 0;
  if ( v2 != 0 )
    return *a1;
  else
    return v3;
}

