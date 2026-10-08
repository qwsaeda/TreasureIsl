/* ea:   0x00016420 */
/* name: sub_16420 */
/* size: 96 */
int __fastcall sub_16420(int *a1, unsigned __int16 *a2)
{
  int v2; // r4
  int v3; // r3

  v2 = *a1;
  v3 = *(_DWORD *)(*a1 + 4);
  while ( v3 != 0 )
  {
    if ( *a2 > (unsigned int)*(unsigned __int16 *)(v3 + 16) )
    {
      v3 = *(_DWORD *)(v3 + 12);
    }
    else
    {
      v2 = v3;
      v3 = *(_DWORD *)(v3 + 8);
    }
  }
  return v2;
}

