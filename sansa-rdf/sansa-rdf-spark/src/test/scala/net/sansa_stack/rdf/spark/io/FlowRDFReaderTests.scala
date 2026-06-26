package net.sansa_stack.rdf.spark.io

import org.apache.jena.datatypes.xsd.XSDDatatype
import org.apache.jena.graph.Triple
import org.apache.spark.SparkConf
import org.apache.spark.sql.SparkSession
import org.scalatest.{BeforeAndAfterAll, FunSuite}

class FlowRDFReaderTests extends FunSuite with BeforeAndAfterAll {

  @transient private var spark: SparkSession = _

  override protected def beforeAll(): Unit = {
    super.beforeAll()
    val conf = new SparkConf()
      .setMaster("local[1]")
      .setAppName("FlowRDFReaderTests")
      .set("spark.ui.enabled", "false")
      .set("spark.serializer", "org.apache.spark.serializer.KryoSerializer")

    spark = SparkSession.builder().config(conf).getOrCreate()
  }

  override protected def afterAll(): Unit = {
    if (spark != null) {
      spark.stop()
      spark = null
    }
    super.afterAll()
  }

  test("parse 2flow lines into Jena triples with numeric literals and URI resources") {
    val input = Seq(
      "Torre_Eiffel -> altura:330",
      "Torre_Eiffel -> localizacao:Paris")

    val triples = FlowRDFReader
      .parseLines(spark.sparkContext.parallelize(input))
      .collect()

    assert(triples.length == 2)
    assert(triples.forall(_.isInstanceOf[Triple]))

    val heightTriple = triples(0)
    assert(heightTriple.getSubject.isURI)
    assert(heightTriple.getSubject.getURI == "http://2flow.org/Torre_Eiffel")
    assert(heightTriple.getPredicate.isURI)
    assert(heightTriple.getPredicate.getURI == "http://2flow.org/altura")
    assert(heightTriple.getObject.isLiteral)
    assert(heightTriple.getObject.getLiteralLexicalForm == "330")
    assert(heightTriple.getObject.getLiteralDatatype == XSDDatatype.XSDinteger)

    val locationTriple = triples(1)
    assert(locationTriple.getSubject.isURI)
    assert(locationTriple.getSubject.getURI == "http://2flow.org/Torre_Eiffel")
    assert(locationTriple.getPredicate.isURI)
    assert(locationTriple.getPredicate.getURI == "http://2flow.org/localizacao")
    assert(locationTriple.getObject.isURI)
    assert(locationTriple.getObject.getURI == "http://2flow.org/Paris")
  }
}
