/* ea:   0x00039798 */
/* name: sub_39798 */
/* size: 136 */
int __fastcall sub_39798(int *a1, unsigned __int8 *a2)
{
  char v3; // r4
  int v4; // r6
  int v5; // r2
  unsigned int v6; // r1

  v3 = 1;
  v4 = *a1;
  v5 = *(_DWORD *)(*a1 + 4);
  if ( v5 != 0 )
  {
    v6 = *a2;
    do
    {
      if ( *(unsigned __int8 *)(v5 + 16) < v6 )
      {
        v5 = *(_DWORD *)(v5 + 12);
      }
      else
      {
        v4 = v5;
        v5 = *(_DWORD *)(v5 + 8);
      }
    }
    while ( v5 != 0 );
  }
  if ( v4 == *a1 )
    return *a1;
  if ( *a2 >= (unsigned int)*(unsigned __int8 *)(v4 + 16) )
    v3 = 0;
  if ( v3 != 0 )
    return *a1;
  else
    return v4;
}

