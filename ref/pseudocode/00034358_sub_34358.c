/* ea:   0x00034358 */
/* name: sub_34358 */
/* size: 84 */
int __fastcall sub_34358(int *a1, unsigned __int8 *a2)
{
  int v2; // r3
  int v3; // r2
  unsigned int v4; // r1

  v2 = *a1;
  v3 = *(_DWORD *)(*a1 + 4);
  if ( v3 != 0 )
  {
    v4 = *a2;
    do
    {
      if ( *(unsigned __int8 *)(v3 + 16) < v4 )
      {
        v3 = *(_DWORD *)(v3 + 12);
      }
      else
      {
        v2 = v3;
        v3 = *(_DWORD *)(v3 + 8);
      }
    }
    while ( v3 != 0 );
  }
  return v2;
}

