/* ea:   0x000290B8 */
/* name: sub_290B8 */
/* size: 64 */
int __fastcall sub_290B8(int a1, int a2, int a3)
{
  if ( a1 == 16 )
    return (int)*(&off_7BB78[4 * a2] + a3);
  if ( a1 == 8 )
    return (int)*(&off_7BBB8[4 * a2] + a3);
  return 0;
}

